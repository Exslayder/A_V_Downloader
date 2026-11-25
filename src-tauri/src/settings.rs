use tauri::{AppHandle, Manager};

use crate::paths;
use crate::state::Settings;

fn default_audio_dir(app: &AppHandle) -> String {
    app.path()
        .audio_dir()
        .or_else(|_| app.path().download_dir())
        .or_else(|_| app.path().home_dir())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn default_video_dir(app: &AppHandle) -> String {
    app.path()
        .video_dir()
        .or_else(|_| app.path().download_dir())
        .or_else(|_| app.path().home_dir())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub fn load(app: &AppHandle) -> Settings {
    let defaults = Settings {
        audio_path: default_audio_dir(app),
        video_path: default_video_dir(app),
        language: "ru".into(),
        auto_update: true,
        ..Settings::default()
    };

    let path = paths::settings_file(app);
    let Ok(raw) = std::fs::read_to_string(&path) else {
        let _ = save(app, &defaults);
        return defaults;
    };

    match serde_json::from_str::<Settings>(&raw) {
        Ok(mut settings) => {
            if settings.audio_path.trim().is_empty() {
                settings.audio_path = defaults.audio_path;
            }
            if settings.video_path.trim().is_empty() {
                settings.video_path = defaults.video_path;
            }
            if settings.language != "ru" && settings.language != "en" {
                settings.language = "ru".into();
            }
            if settings.theme != "light" && settings.theme != "dark" && settings.theme != "system" {
                settings.theme = "system".into();
            }
            settings
        }
        Err(_) => {
            let _ = save(app, &defaults);
            defaults
        }
    }
}

pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let path = paths::settings_file(app);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}
