use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

fn is_writable(dir: &Path) -> bool {
    if std::fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(".write-test");
    match std::fs::write(&probe, b"ok") {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// Prefers a `data` folder next to the executable (portable install) and falls
/// back to `%APPDATA%` when the program directory is read-only, for example
/// when AVDock is installed into `Program Files`.
pub fn data_root(app: &AppHandle) -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join("data");
            if is_writable(&candidate) {
                return candidate;
            }
        }
    }
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn settings_file(app: &AppHandle) -> PathBuf {
    data_root(app).join("settings.json")
}

pub fn bin_dir(app: &AppHandle) -> PathBuf {
    data_root(app).join("bin")
}

pub fn tmp_dir(app: &AppHandle) -> PathBuf {
    data_root(app).join("tmp")
}

/// Copies settings and yt-dlp from the previous `%APPDATA%` layout, if present.
pub fn migrate_legacy(app: &AppHandle) {
    let Ok(legacy) = app.path().app_data_dir() else {
        return;
    };
    let root = data_root(app);
    if legacy == root {
        return;
    }

    let legacy_settings = legacy.join("settings.json");
    let new_settings = root.join("settings.json");
    if !new_settings.exists() && legacy_settings.is_file() {
        if let Some(parent) = new_settings.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::copy(&legacy_settings, &new_settings);
    }

    let legacy_ytdlp = legacy.join("bin").join("yt-dlp.exe");
    let new_ytdlp = root.join("bin").join("yt-dlp.exe");
    if !new_ytdlp.exists() && legacy_ytdlp.is_file() {
        if let Some(parent) = new_ytdlp.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::copy(&legacy_ytdlp, &new_ytdlp);
    }
}

pub fn ensure_tmp(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = tmp_dir(app);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}
