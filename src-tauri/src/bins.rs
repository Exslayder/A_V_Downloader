use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri::path::BaseDirectory;
use tokio::io::AsyncWriteExt;

use crate::state::AppState;

pub const YTDLP_LATEST_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";
pub const YTDLP_API_URL: &str = "https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YtdlpStatus {
    pub state: String,
    pub version: Option<String>,
    pub percent: Option<f64>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinariesInfo {
    pub ytdlp_path: Option<String>,
    pub ytdlp_version: Option<String>,
    pub ffmpeg_path: Option<String>,
    pub ffmpeg_version: Option<String>,
}

fn hidden(cmd: &mut std::process::Command) -> &mut std::process::Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    cmd
}

pub fn app_bin_dir(app: &AppHandle) -> PathBuf {
    crate::paths::bin_dir(app)
}

pub fn ytdlp_exe(app: &AppHandle) -> PathBuf {
    app_bin_dir(app).join("yt-dlp.exe")
}

fn path_lookup(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub fn ffmpeg_exe(app: &AppHandle) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(p) = app.path().resolve("resources/ffmpeg.exe", BaseDirectory::Resource) {
        candidates.push(p);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("resources").join("ffmpeg.exe"));
            candidates.push(dir.join("ffmpeg.exe"));
        }
    }
    candidates.push(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("ffmpeg.exe"),
    );

    for candidate in candidates {
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    path_lookup("ffmpeg.exe")
}

pub fn exe_version(path: &Path, args: &[&str]) -> Option<String> {
    let mut cmd = std::process::Command::new(path);
    cmd.args(args);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    hidden(&mut cmd);
    let output = cmd.output().ok()?;
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    if text.trim().is_empty() {
        text = String::from_utf8_lossy(&output.stderr).to_string();
    }
    let line = text.lines().find(|l| !l.trim().is_empty())?.trim().to_string();
    if line.is_empty() {
        None
    } else {
        Some(line)
    }
}

/// Kills a process together with its children (ffmpeg spawned by yt-dlp).
pub fn kill_tree(pid: u32) {
    let mut cmd = std::process::Command::new("taskkill");
    cmd.args(["/PID", &pid.to_string(), "/T", "/F"]);
    cmd.stdout(Stdio::null()).stderr(Stdio::null());
    hidden(&mut cmd);
    let _ = cmd.status();
}

pub fn ytdlp_version(path: &Path) -> Option<String> {
    exe_version(path, &["--version"])
}

pub fn ffmpeg_version(path: &Path) -> Option<String> {
    let line = exe_version(path, &["-version"])?;
    // "ffmpeg version 8.0.1-essentials_build ..." -> "8.0.1"
    let parts: Vec<&str> = line.split_whitespace().collect();
    parts
        .get(2)
        .map(|s| s.split('-').next().unwrap_or(s).to_string())
        .or(Some(line))
}

pub fn binaries_info(app: &AppHandle) -> BinariesInfo {
    let ytdlp = ytdlp_exe(app);
    let ytdlp_path = ytdlp.is_file().then(|| ytdlp.to_string_lossy().to_string());
    let ytdlp_version = ytdlp_path.as_ref().and_then(|_| ytdlp_version(&ytdlp));

    let ffmpeg = ffmpeg_exe(app);
    let ffmpeg_path = ffmpeg.as_ref().map(|p| p.to_string_lossy().to_string());
    let ffmpeg_version = ffmpeg.as_ref().and_then(|p| ffmpeg_version(p));

    BinariesInfo {
        ytdlp_path,
        ytdlp_version,
        ffmpeg_path,
        ffmpeg_version,
    }
}

pub fn emit_status(app: &AppHandle, status: YtdlpStatus) {
    let _ = app.emit("ytdlp-status", status);
}

pub fn ready_status(app: &AppHandle) -> YtdlpStatus {
    let path = ytdlp_exe(app);
    if path.is_file() {
        YtdlpStatus {
            state: "ready".into(),
            version: ytdlp_version(&path),
            percent: None,
            error: None,
        }
    } else {
        YtdlpStatus {
            state: "missing".into(),
            version: None,
            percent: None,
            error: None,
        }
    }
}

pub async fn latest_ytdlp_version() -> Option<String> {
    let client = reqwest::Client::builder()
        .user_agent("AVDock")
        .timeout(Duration::from_secs(10))
        .build()
        .ok()?;
    let response = client.get(YTDLP_API_URL).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let json: serde_json::Value = response.json().await.ok()?;
    json.get("tag_name")?.as_str().map(|s| s.to_string())
}

pub async fn install_ytdlp(app: &AppHandle) -> Result<String, String> {
    {
        let state = app.state::<AppState>();
        if state.ytdlp_busy.swap(true, Ordering::SeqCst) {
            return Err("busy".into());
        }
    }

    let result = install_ytdlp_inner(app).await;

    {
        let state = app.state::<AppState>();
        state.ytdlp_busy.store(false, Ordering::SeqCst);
    }

    match &result {
        Ok(version) => emit_status(
            app,
            YtdlpStatus {
                state: "ready".into(),
                version: Some(version.clone()),
                percent: Some(100.0),
                error: None,
            },
        ),
        Err(error) => emit_status(
            app,
            YtdlpStatus {
                state: "error".into(),
                version: None,
                percent: None,
                error: Some(error.clone()),
            },
        ),
    }

    result
}

async fn install_ytdlp_inner(app: &AppHandle) -> Result<String, String> {
    let dir = app_bin_dir(app);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let target = dir.join("yt-dlp.exe");
    let temp = dir.join("yt-dlp.new.exe");

    emit_status(
        app,
        YtdlpStatus {
            state: "downloading".into(),
            version: None,
            percent: Some(0.0),
            error: None,
        },
    );

    let client = reqwest::Client::builder()
        .user_agent("AVDock")
        .connect_timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(YTDLP_LATEST_URL)
        .send()
        .await
        .map_err(|e| format!("network: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("http {}", response.status()));
    }

    let total = response.content_length();
    let mut file = tokio::fs::File::create(&temp)
        .await
        .map_err(|e| e.to_string())?;
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut last_emit = Instant::now();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        if last_emit.elapsed() > Duration::from_millis(150) {
            last_emit = Instant::now();
            let percent = total
                .filter(|t| *t > 0)
                .map(|t| (downloaded as f64 / t as f64 * 100.0).min(100.0));
            emit_status(
                app,
                YtdlpStatus {
                    state: "downloading".into(),
                    version: None,
                    percent,
                    error: None,
                },
            );
        }
    }

    file.flush().await.map_err(|e| e.to_string())?;
    drop(file);

    if let Err(first) = tokio::fs::rename(&temp, &target).await {
        // A running yt-dlp.exe can lock the target; try to replace it once more.
        let _ = tokio::fs::remove_file(&target).await;
        tokio::fs::rename(&temp, &target).await.map_err(|_| {
            let _ = std::fs::remove_file(&temp);
            first.to_string()
        })?;
    }

    let version = ytdlp_version(&target).ok_or_else(|| "downloaded file is not executable".to_string())?;
    Ok(version)
}

impl Default for YtdlpStatus {
    fn default() -> Self {
        Self {
            state: "missing".into(),
            version: None,
            percent: None,
            error: None,
        }
    }
}
