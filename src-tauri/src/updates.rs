use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::UpdaterExt;

use crate::bins;
use crate::state::AppState;

#[derive(Clone, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateReport {
    pub ytdlp_current: Option<String>,
    pub ytdlp_latest: Option<String>,
    pub app_current: String,
    pub app_latest: Option<String>,
    pub app_error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateStatus {
    pub phase: String,
    pub version: Option<String>,
    pub percent: Option<f64>,
    pub error: Option<String>,
}

fn emit(app: &AppHandle, status: AppUpdateStatus) {
    let _ = app.emit("app-update-status", status);
}

pub async fn check_all(app: &AppHandle) -> UpdateReport {
    let ytdlp_path = bins::ytdlp_exe(app);
    let ytdlp_current = bins::ytdlp_version(&ytdlp_path);
    let ytdlp_latest = bins::latest_ytdlp_version().await;

    let mut report = UpdateReport {
        ytdlp_current,
        ytdlp_latest,
        app_current: app.package_info().version.to_string(),
        app_latest: None,
        app_error: None,
    };

    match app.updater() {
        Ok(updater) => match updater.check().await {
            Ok(Some(update)) => report.app_latest = Some(update.version.clone()),
            Ok(None) => {}
            Err(error) => report.app_error = Some(error.to_string()),
        },
        Err(error) => report.app_error = Some(error.to_string()),
    }

    report
}

pub async fn install_app_update(app: &AppHandle) -> Result<(), String> {
    install_inner(app, false).await
}

async fn install_inner(app: &AppHandle, silent: bool) -> Result<(), String> {
    emit(
        app,
        AppUpdateStatus {
            phase: "checking".into(),
            ..Default::default()
        },
    );

    let updater = match app.updater() {
        Ok(updater) => updater,
        Err(error) => return fail(app, silent, error.to_string()),
    };

    let update = match updater.check().await {
        Ok(Some(update)) => update,
        Ok(None) => {
            emit(
                app,
                AppUpdateStatus {
                    phase: "none".into(),
                    ..Default::default()
                },
            );
            return Err("no_update".into());
        }
        Err(error) => return fail(app, silent, error.to_string()),
    };

    let version = update.version.clone();
    let downloaded = Arc::new(AtomicU64::new(0));
    let total = Arc::new(AtomicU64::new(0));

    emit(
        app,
        AppUpdateStatus {
            phase: "downloading".into(),
            version: Some(version.clone()),
            percent: Some(0.0),
            error: None,
        },
    );

    let status_app = app.clone();
    let status_version = version.clone();
    let counter = downloaded.clone();
    let total_counter = total.clone();
    let mut last_emit = Instant::now();

    let result = update
        .download_and_install(
            move |chunk_length, content_length| {
                let done = counter.fetch_add(chunk_length as u64, Ordering::SeqCst)
                    + chunk_length as u64;
                if let Some(length) = content_length {
                    total_counter.store(length, Ordering::SeqCst);
                }
                if last_emit.elapsed() > Duration::from_millis(150) {
                    last_emit = Instant::now();
                    let all = total_counter.load(Ordering::SeqCst);
                    let percent = if all > 0 {
                        Some((done as f64 / all as f64 * 100.0).min(100.0))
                    } else {
                        None
                    };
                    emit(
                        &status_app,
                        AppUpdateStatus {
                            phase: "downloading".into(),
                            version: Some(status_version.clone()),
                            percent,
                            error: None,
                        },
                    );
                }
            },
            move || {},
        )
        .await;

    if let Err(error) = result {
        return fail(app, silent, error.to_string());
    }

    emit(
        app,
        AppUpdateStatus {
            phase: "installing".into(),
            version: Some(version),
            percent: Some(100.0),
            error: None,
        },
    );

    // On Windows `download_and_install` exits the app after launching the
    // installer. Other platforms need an explicit relaunch.
    #[cfg(not(windows))]
    app.restart();

    Ok(())
}

fn fail(app: &AppHandle, silent: bool, message: String) -> Result<(), String> {
    if !silent {
        emit(
            app,
            AppUpdateStatus {
                phase: "error".into(),
                error: Some(message.clone()),
                ..Default::default()
            },
        );
    }
    Err(message)
}

/// Ensures yt-dlp is present and up to date, then installs AVDock updates when
/// the user allowed automatic updates.
pub fn spawn_startup(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;

        let auto_update = {
            let state = app.state::<AppState>();
            let settings = state.settings.lock().unwrap();
            settings.auto_update
        };

        let path = bins::ytdlp_exe(&app);
        let current = bins::ytdlp_version(&path);
        bins::emit_status(&app, bins::ready_status(&app));

        if current.is_none() {
            let _ = bins::install_ytdlp(&app).await;
        } else if auto_update {
            if let Some(latest) = bins::latest_ytdlp_version().await {
                if current.as_deref() != Some(latest.as_str()) {
                    let _ = bins::install_ytdlp(&app).await;
                }
            }
        }

        // A debug build is not signed with the release key, so skip the check
        // during development to keep the logs clean.
        if auto_update && !cfg!(debug_assertions) {
            let _ = install_inner(&app, true).await;
        }
    });
}
