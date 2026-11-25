use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use tauri::{AppHandle, Manager};

use crate::bins;
use crate::state::{update_item, AppState, ItemStatus, MediaKind};

const SEP: char = '\u{1f}';

#[derive(Default)]
struct ErrorSink {
    count: usize,
    reasons: Vec<String>,
}

pub async fn run_item(app: AppHandle, id: u64) {
    let state = app.state::<AppState>();
    state.cancel.store(false, Ordering::SeqCst);
    *state.current_id.lock().unwrap() = Some(id);

    let (url, kind) = {
        let queue = state.queue.lock().unwrap();
        match queue.iter().find(|i| i.id == id) {
            Some(item) => (item.url.clone(), item.kind),
            None => return,
        }
    };

    let result = download(&app, id, &url, kind).await;

    let canceled = {
        let state = app.state::<AppState>();
        state.cancel.load(Ordering::SeqCst)
    };

    {
        let state = app.state::<AppState>();
        *state.current_id.lock().unwrap() = None;
        *state.current_pid.lock().unwrap() = None;
        state.cancel.store(false, Ordering::SeqCst);
    }

    match result {
        Ok(DownloadOutcome { warn }) => {
            update_item(&app, id, true, |item| {
                item.status = if canceled {
                    ItemStatus::Canceled
                } else {
                    ItemStatus::Done
                };
                item.percent = if item.status == ItemStatus::Done { 100.0 } else { item.percent };
                item.speed = None;
                item.eta = None;
                item.warn = warn;
            });
        }
        Err(error) => {
            update_item(&app, id, true, |item| {
                item.status = if canceled {
                    ItemStatus::Canceled
                } else {
                    ItemStatus::Error
                };
                item.speed = None;
                item.eta = None;
                if !canceled {
                    item.error = Some(error);
                }
            });
        }
    }
}

struct DownloadOutcome {
    warn: Option<String>,
}

async fn download(
    app: &AppHandle,
    id: u64,
    url: &str,
    kind: MediaKind,
) -> Result<DownloadOutcome, String> {
    let state = app.state::<AppState>();

    let (audio_path, video_path) = {
        let settings = state.settings.lock().unwrap();
        (settings.audio_path.clone(), settings.video_path.clone())
    };

    let target_dir = if kind == MediaKind::Mp3 {
        audio_path
    } else {
        video_path
    };

    if target_dir.trim().is_empty() {
        return Err("output_dir_empty".into());
    }

    let target_dir = PathBuf::from(&target_dir);
    std::fs::create_dir_all(&target_dir)
        .map_err(|e| format!("output_dir: {e}"))?;

    let ytdlp = bins::ytdlp_exe(app);
    if !ytdlp.is_file() {
        return Err("ytdlp_missing".into());
    }

    let ffmpeg = bins::ffmpeg_exe(app).ok_or_else(|| "ffmpeg_missing".to_string())?;
    let ffmpeg_dir = ffmpeg.parent().unwrap_or(Path::new(".")).to_path_buf();
    let tmp_dir = crate::paths::ensure_tmp(app)?;

    let outtmpl = match kind {
        MediaKind::Mp3 => "%(title)s - %(uploader)s.%(ext)s",
        _ => "%(title)s.%(ext)s",
    };
    let outtmpl = target_dir.join(outtmpl);

    let progress_template = format!(
        "download:AVPROG{SEP}%(progress.status)j{SEP}%(progress.downloaded_bytes)j{SEP}%(progress.total_bytes)j{SEP}%(progress.total_bytes_estimate)j{SEP}%(progress.speed)j{SEP}%(progress.eta)j{SEP}%(progress._percent_str)j"
    );
    let meta_template = format!(
        "before_dl:AVMETA{SEP}%(playlist_index)j{SEP}%(n_entries)j{SEP}%(title)j"
    );
    let file_template = format!("after_move:AVFILE{SEP}%(filepath)j");

    let mut args: Vec<String> = vec![
        "--newline".into(),
        "--no-colors".into(),
        "--encoding".into(),
        "utf-8".into(),
        "--ignore-errors".into(),
        "--no-warnings".into(),
        "--no-simulate".into(),
        "--progress".into(),
        "--force-overwrites".into(),
        "--ffmpeg-location".into(),
        ffmpeg_dir.to_string_lossy().to_string(),
        "--paths".into(),
        format!("temp:{}", tmp_dir.to_string_lossy()),
        "--progress-template".into(),
        progress_template,
        "--print".into(),
        meta_template,
        "--print".into(),
        file_template,
        "-o".into(),
        outtmpl.to_string_lossy().to_string(),
        url.to_string(),
    ];

    // Insert format options before the URL (argument order does not matter for
    // yt-dlp, but keeping the URL last reads better in logs).
    let url_arg = args.pop().unwrap();
    match kind {
        MediaKind::Mp4 => {
            args.push("--format".into());
            args.push("bv*[ext=mp4]+ba[ext=m4a]/bv*+ba/best".into());
            args.push("--merge-output-format".into());
            args.push("mp4".into());
        }
        MediaKind::Webm => {
            args.push("--format".into());
            args.push("bv*[ext=webm]+ba[ext=webm]/bv*+ba/best".into());
            args.push("--merge-output-format".into());
            args.push("webm".into());
        }
        MediaKind::Mp3 => {
            args.push("--format".into());
            args.push("bestaudio/best".into());
            args.push("--extract-audio".into());
            args.push("--audio-format".into());
            args.push("mp3".into());
            args.push("--audio-quality".into());
            args.push("320K".into());
            args.push("--write-thumbnail".into());
            args.push("--convert-thumbnails".into());
            args.push("jpg".into());
            args.push("--embed-thumbnail".into());
            args.push("--embed-metadata".into());
            args.push("--ppa".into());
            args.push(
                "ffmpeg:-id3v2_version 3 -metadata:s:v title=\"Album cover\" -metadata:s:v comment=\"Cover (Front)\""
                    .into(),
            );
        }
    }
    args.push(url_arg);

    let mut command = Command::new(&ytdlp);
    command.args(&args);
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
        *state.current_pid.lock().unwrap() = child.id();
    }

    let errors = Arc::new(Mutex::new(ErrorSink::default()));
    let outputs = Arc::new(Mutex::new(Vec::<String>::new()));

    let stdout = child.stdout.take().ok_or("no stdout")?;
    let stderr = child.stderr.take().ok_or("no stderr")?;

    let app_out = app.clone();
    let errors_out = errors.clone();
    let outputs_out = outputs.clone();
    let stdout_task = tauri::async_runtime::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            handle_line(&app_out, id, &line, &errors_out, &outputs_out, false);
        }
    });

    let app_err = app.clone();
    let errors_err = errors.clone();
    let outputs_err = outputs.clone();
    let stderr_task = tauri::async_runtime::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            handle_line(&app_err, id, &line, &errors_err, &outputs_err, true);
        }
    });

    let status = child.wait().await;
    let _ = stdout_task.await;
    let _ = stderr_task.await;

    let code = status.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);

    let produced: Vec<String> = outputs.lock().unwrap().clone();
    let (error_count, reasons) = {
        let sink = errors.lock().unwrap();
        (sink.count, sink.reasons.clone())
    };

    if kind == MediaKind::Mp3 && !produced.is_empty() {
        cleanup_orphan_thumbnails(&produced);
    }

    if let Some(last) = produced.last() {
        update_item(app, id, true, |item| {
            item.output_path = Some(last.clone());
        });
    }

    if error_count > 0 {
        let mut unique: Vec<String> = Vec::new();
        for reason in reasons {
            if !unique.contains(&reason) {
                unique.push(reason);
            }
        }
        let warn = format!("{}|{}", error_count, unique.join(", "));
        if produced.is_empty() {
            return Err(warn);
        }
        return Ok(DownloadOutcome { warn: Some(warn) });
    }

    if produced.is_empty() && code != 0 {
        return Err(format!("exit_code: {code}"));
    }

    Ok(DownloadOutcome { warn: None })
}

fn handle_line(
    app: &AppHandle,
    id: u64,
    line: &str,
    errors: &Arc<Mutex<ErrorSink>>,
    outputs: &Arc<Mutex<Vec<String>>>,
    is_stderr: bool,
) {
    let line = line.trim_end_matches('\r');

    if let Some(rest) = line.strip_prefix("AVPROG") {
        let rest = rest.trim_start_matches(SEP);
        let fields: Vec<&str> = rest.split(SEP).collect();
        if fields.len() < 7 {
            return;
        }
        let status = json_str(fields[0]).unwrap_or_default();
        let downloaded = json_u64(fields[1]);
        let total = json_u64(fields[2]).or_else(|| json_u64(fields[3]));
        let speed = json_f64(fields[4]);
        let eta = json_u64(fields[5]);
        let percent_fallback = json_str(fields[6])
            .and_then(|s| s.trim().trim_end_matches('%').parse::<f64>().ok());

        update_item(app, id, false, |item| {
            item.downloaded = downloaded;
            item.total = total;
            item.speed = speed;
            item.eta = eta;
            if status == "finished" {
                item.percent = 100.0;
            } else if let (Some(done), Some(all)) = (downloaded, total) {
                if all > 0 {
                    item.percent = (done as f64 / all as f64 * 100.0).min(100.0);
                }
            } else if let Some(percent) = percent_fallback {
                item.percent = percent.clamp(0.0, 100.0);
            }
        });
        return;
    }

    if let Some(rest) = line.strip_prefix("AVMETA") {
        let rest = rest.trim_start_matches(SEP);
        let fields: Vec<&str> = rest.split(SEP).collect();
        if fields.len() < 3 {
            return;
        }
        let playlist_index = json_u64(fields[0]);
        let n_entries = json_u64(fields[1]);
        let title = json_str(fields[2]);
        update_item(app, id, false, |item| {
            if title.is_some() {
                item.title = title;
            }
            item.playlist_index = playlist_index;
            item.n_entries = n_entries;
        });
        return;
    }

    if let Some(rest) = line.strip_prefix("AVFILE") {
        let rest = rest.trim_start_matches(SEP);
        if let Some(path) = json_str(rest) {
            if !path.is_empty() {
                outputs.lock().unwrap().push(path.clone());
                update_item(app, id, false, |item| {
                    item.output_path = Some(path);
                });
            }
        }
        return;
    }

    if is_stderr {
        if let Some(reason) = classify_error(line) {
            let mut sink = errors.lock().unwrap();
            sink.count += 1;
            sink.reasons.push(reason);
            return;
        }
        if line.starts_with("[ExtractAudio]")
            || line.starts_with("[EmbedThumbnail]")
            || line.starts_with("[Metadata]")
            || line.starts_with("[Merger]")
            || line.starts_with("[VideoConvertor]")
        {
            update_item(app, id, false, |item| {
                if item.status == ItemStatus::Downloading {
                    item.status = ItemStatus::Processing;
                }
            });
        }
    }
}

fn json_str(raw: &str) -> Option<String> {
    match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(serde_json::Value::String(s)) => Some(s),
        Ok(serde_json::Value::Null) => None,
        Ok(other) => Some(other.to_string()),
        Err(_) => None,
    }
}

fn json_u64(raw: &str) -> Option<u64> {
    match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(value) => value.as_u64().or_else(|| value.as_f64().map(|f| f as u64)),
        Err(_) => None,
    }
}

fn json_f64(raw: &str) -> Option<f64> {
    match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(value) => value.as_f64(),
        Err(_) => None,
    }
}

fn classify_error(line: &str) -> Option<String> {
    if line.contains("Sign in to confirm your age") {
        return Some("age_restricted".into());
    }
    if line.contains("is unavailable") || line.contains("private video") || line.contains("Private video") {
        return Some("unavailable".into());
    }
    if let Some(pos) = line.find("ERROR:") {
        let tail = line[pos + "ERROR:".len()..].trim();
        let short = tail.split('.').next().unwrap_or(tail).trim();
        let short = if short.is_empty() { "unknown" } else { short };
        return Some(format!("raw:{short}"));
    }
    None
}

fn cleanup_orphan_thumbnails(produced: &[String]) {
    let images = ["jpg", "jpeg", "webp", "png"];
    for file in produced {
        let path = Path::new(file);
        let Some(stem) = path.file_stem() else { continue };
        let Some(parent) = path.parent() else { continue };
        for ext in images {
            let candidate = parent.join(format!("{}.{ext}", stem.to_string_lossy()));
            if candidate != path && candidate.is_file() {
                let _ = std::fs::remove_file(candidate);
            }
        }
    }
}
