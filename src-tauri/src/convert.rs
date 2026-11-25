use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::Ordering;

use serde::Serialize;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use tauri::{AppHandle, Emitter, Manager};

use crate::bins;
use crate::state::AppState;

pub const TARGETS: [&str; 7] = ["mp4", "mkv", "webm", "mov", "mp3", "m4a", "wav"];

#[derive(Clone, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConvertProgress {
    pub running: bool,
    pub phase: String,
    pub percent: f64,
    pub speed: Option<String>,
    pub file: Option<String>,
    pub output: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Encoder {
    Nvenc,
    X264,
    Vp9,
    Mp3,
    Aac,
    Wav,
}

impl Encoder {
    fn phase(self) -> &'static str {
        match self {
            Encoder::Nvenc => "nvenc",
            Encoder::X264 => "cpu",
            Encoder::Vp9 => "vp9",
            Encoder::Mp3 | Encoder::Aac | Encoder::Wav => "audio",
        }
    }
}

struct Probe {
    duration: f64,
    has_video: bool,
    has_audio: bool,
}

fn emit(app: &AppHandle, progress: ConvertProgress) {
    let _ = app.emit("convert-progress", progress);
}

fn is_video_target(target: &str) -> bool {
    matches!(target, "mp4" | "mkv" | "webm" | "mov")
}

fn is_h264_target(target: &str) -> bool {
    matches!(target, "mp4" | "mkv" | "mov")
}

fn audio_encoder(target: &str) -> Option<Encoder> {
    match target {
        "mp3" => Some(Encoder::Mp3),
        "m4a" => Some(Encoder::Aac),
        "wav" => Some(Encoder::Wav),
        _ => None,
    }
}

pub fn spawn_convert(app: AppHandle, input: String, target: String) {
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        if state.convert_busy.swap(true, Ordering::SeqCst) {
            emit(
                &app,
                ConvertProgress {
                    running: false,
                    phase: "error".into(),
                    error: Some("busy".into()),
                    ..Default::default()
                },
            );
            return;
        }

        let result = convert_inner(&app, &input, &target).await;

        {
            let state = app.state::<AppState>();
            state.convert_busy.store(false, Ordering::SeqCst);
            *state.convert_pid.lock().unwrap() = None;
        }

        match result {
            Ok(output) => emit(
                &app,
                ConvertProgress {
                    running: false,
                    phase: "done".into(),
                    percent: 100.0,
                    file: Some(input),
                    output: Some(output),
                    ..Default::default()
                },
            ),
            Err(error) => {
                let canceled = error == "canceled";
                emit(
                    &app,
                    ConvertProgress {
                        running: false,
                        phase: if canceled { "canceled".into() } else { "error".into() },
                        file: Some(input),
                        error: Some(error),
                        ..Default::default()
                    },
                );
            }
        }
    });
}

pub fn cancel(app: &AppHandle) {
    let state = app.state::<AppState>();
    let pid = state.convert_pid.lock().unwrap().take();
    if let Some(pid) = pid {
        bins::kill_tree(pid);
    }
}

async fn convert_inner(app: &AppHandle, input: &str, target: &str) -> Result<String, String> {
    let target = target.to_lowercase();
    if !TARGETS.contains(&target.as_str()) {
        return Err("bad_target".into());
    }

    let input_path = PathBuf::from(input);
    if !input_path.is_file() {
        return Err("file_not_found".into());
    }

    let extension = input_path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if extension == target {
        return Err("same_format".into());
    }

    let output_path = input_path.with_extension(&target);
    if output_path == input_path {
        return Err("same_format".into());
    }

    let ffmpeg = bins::ffmpeg_exe(app).ok_or_else(|| "ffmpeg_missing".to_string())?;
    let probe = probe_media(&ffmpeg, &input_path).await;

    let video_target = is_video_target(&target);
    if video_target && !probe.has_video {
        return Err("no_video".into());
    }
    if !video_target && !probe.has_audio {
        return Err("no_audio".into());
    }

    let output = output_path.to_string_lossy().to_string();

    emit(
        app,
        ConvertProgress {
            running: true,
            phase: if is_h264_target(&target) {
                "nvenc".into()
            } else if target == "webm" {
                "vp9".into()
            } else {
                "audio".into()
            },
            percent: 0.0,
            file: Some(input.to_string()),
            output: Some(output.clone()),
            ..Default::default()
        },
    );

    if is_h264_target(&target) {
        let nvenc = run_ffmpeg(
            app,
            &ffmpeg,
            &input_path,
            &output_path,
            probe.duration,
            Encoder::Nvenc,
        )
        .await?;

        if nvenc != 0 {
            emit(
                app,
                ConvertProgress {
                    running: true,
                    phase: "cpu".into(),
                    percent: 0.0,
                    file: Some(input.to_string()),
                    output: Some(output.clone()),
                    ..Default::default()
                },
            );
            let cpu = run_ffmpeg(
                app,
                &ffmpeg,
                &input_path,
                &output_path,
                probe.duration,
                Encoder::X264,
            )
            .await?;
            if cpu != 0 {
                return Err("ffmpeg_failed".into());
            }
        }
    } else {
        let encoder = if target == "webm" {
            Encoder::Vp9
        } else {
            audio_encoder(&target).ok_or_else(|| "bad_target".to_string())?
        };

        let code = run_ffmpeg(
            app,
            &ffmpeg,
            &input_path,
            &output_path,
            probe.duration,
            encoder,
        )
        .await?;
        if code != 0 {
            return Err("ffmpeg_failed".into());
        }
    }

    Ok(output)
}

async fn probe_media(ffmpeg: &Path, input: &Path) -> Probe {
    let mut command = Command::new(ffmpeg);
    command.arg("-i").arg(input);
    command.stdout(Stdio::null()).stderr(Stdio::piped());
    #[cfg(windows)]
    {
        command.creation_flags(0x0800_0000);
    }

    let fallback = Probe {
        duration: 0.0,
        has_video: true,
        has_audio: true,
    };

    let Ok(output) = command.output().await else {
        return fallback;
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut duration = 0.0;
    let mut has_video = false;
    let mut has_audio = false;

    for line in stderr.lines() {
        if duration == 0.0 {
            if let Some(pos) = line.find("Duration:") {
                let time = line[pos + "Duration:".len()..]
                    .split(',')
                    .next()
                    .unwrap_or("")
                    .trim();
                if let Some(parsed) = parse_timestamp(time) {
                    duration = parsed;
                }
            }
        }
        if line.contains("Stream #") {
            if line.contains(" Video:") {
                has_video = true;
            }
            if line.contains(" Audio:") {
                has_audio = true;
            }
        }
    }

    if !has_video && !has_audio {
        return fallback;
    }

    Probe {
        duration,
        has_video,
        has_audio,
    }
}

fn parse_timestamp(time: &str) -> Option<f64> {
    let mut parts = time.split(':');
    let hours: f64 = parts.next()?.parse().ok()?;
    let minutes: f64 = parts.next()?.parse().ok()?;
    let seconds: f64 = parts.next()?.parse().ok()?;
    Some(hours * 3600.0 + minutes * 60.0 + seconds)
}

async fn run_ffmpeg(
    app: &AppHandle,
    ffmpeg: &Path,
    input: &Path,
    output: &Path,
    duration: f64,
    encoder: Encoder,
) -> Result<i32, String> {
    let phase = encoder.phase();
    let mut command = Command::new(ffmpeg);
    command.arg("-y");

    if encoder == Encoder::Nvenc {
        command.args(["-hwaccel", "cuda"]);
    }

    command.arg("-i").arg(input);

    match encoder {
        Encoder::Nvenc => {
            command.args([
                "-c:v",
                "h264_nvenc",
                "-preset",
                "p1",
                "-rc",
                "vbr",
                "-cq",
                "18",
                "-c:a",
                "aac",
                "-b:a",
                "192k",
            ]);
        }
        Encoder::X264 => {
            command.args([
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-crf",
                "18",
                "-c:a",
                "aac",
                "-b:a",
                "192k",
            ]);
        }
        Encoder::Vp9 => {
            command.args([
                "-c:v",
                "libvpx-vp9",
                "-crf",
                "30",
                "-b:v",
                "0",
                "-row-mt",
                "1",
                "-cpu-used",
                "4",
                "-c:a",
                "libopus",
                "-b:a",
                "128k",
            ]);
        }
        Encoder::Mp3 => {
            command.args(["-vn", "-c:a", "libmp3lame", "-b:a", "320k"]);
        }
        Encoder::Aac => {
            command.args(["-vn", "-c:a", "aac", "-b:a", "256k"]);
        }
        Encoder::Wav => {
            command.args(["-vn", "-c:a", "pcm_s16le"]);
        }
    }

    command.args(["-progress", "pipe:1", "-nostats"]);
    command.arg(output);
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    #[cfg(windows)]
    {
        command.creation_flags(0x0800_0000);
    }

    let mut child = command.spawn().map_err(|e| format!("spawn: {e}"))?;

    {
        let state = app.state::<AppState>();
        *state.convert_pid.lock().unwrap() = child.id();
    }

    let stdout = child.stdout.take().ok_or("no stdout")?;
    let stderr = child.stderr.take().ok_or("no stderr")?;

    let app_out = app.clone();
    let stdout_task = tauri::async_runtime::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        let mut out_time_us: u64 = 0;
        let mut speed = String::new();
        while let Ok(Some(line)) = lines.next_line().await {
            let line = line.trim();
            if let Some(value) = line.strip_prefix("out_time_ms=") {
                out_time_us = value.parse().unwrap_or(0);
            } else if let Some(value) = line.strip_prefix("speed=") {
                speed = value.trim().to_string();
            } else if line.starts_with("progress=") {
                let current = out_time_us as f64 / 1_000_000.0;
                let percent = if duration > 0.0 {
                    (current / duration * 100.0).clamp(0.0, 100.0)
                } else {
                    0.0
                };
                emit(
                    &app_out,
                    ConvertProgress {
                        running: true,
                        phase: phase.to_string(),
                        percent,
                        speed: if speed.is_empty() { None } else { Some(speed.clone()) },
                        ..Default::default()
                    },
                );
            }
        }
    });

    let app_err = app.clone();
    let stderr_task = tauri::async_runtime::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        let mut tail: Vec<String> = Vec::new();
        while let Ok(Some(line)) = lines.next_line().await {
            if !line.trim().is_empty() {
                tail.push(line);
                if tail.len() > 6 {
                    tail.remove(0);
                }
            }
        }
        if !tail.is_empty() {
            let _ = app_err.emit("convert-log", tail.join("\n"));
        }
    });

    let status = child.wait().await;
    let _ = stdout_task.await;
    let _ = stderr_task.await;

    Ok(status.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1))
}
