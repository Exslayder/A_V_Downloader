#![cfg_attr(windows, windows_subsystem = "windows")]

mod bins;
mod convert;
mod download;
mod paths;
mod queue;
mod settings;
mod state;
mod updates;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use state::{AppState, ItemStatus, MediaKind, QueueItem, Settings};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    version: String,
    identifier: String,
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
fn save_settings(app: AppHandle, state: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    let mut current = state.settings.lock().unwrap();
    let mut next = settings;
    if next.audio_path.trim().is_empty() {
        next.audio_path = current.audio_path.clone();
    }
    if next.video_path.trim().is_empty() {
        next.video_path = current.video_path.clone();
    }
    if next.language != "ru" && next.language != "en" {
        next.language = "ru".into();
    }
    if next.theme != "light" && next.theme != "dark" && next.theme != "system" {
        next.theme = "system".into();
    }
    *current = next.clone();
    drop(current);
    apply_window_theme(&app, &next.theme);
    settings::save(&app, &next)
}

fn apply_window_theme(app: &AppHandle, theme: &str) {
    if let Some(window) = app.get_webview_window("main") {
        let value = match theme {
            "dark" => Some(tauri::Theme::Dark),
            "light" => Some(tauri::Theme::Light),
            _ => None,
        };
        let _ = window.set_theme(value);
    }
}

#[tauri::command]
fn get_queue(state: State<'_, AppState>) -> Vec<QueueItem> {
    state.queue.lock().unwrap().clone()
}

#[tauri::command]
fn enqueue(app: AppHandle, state: State<'_, AppState>, urls: Vec<String>, kind: MediaKind) -> Result<Vec<u64>, String> {
    let mut ids = Vec::new();
    let now = state::now_millis();
    {
        let mut queue = state.queue.lock().unwrap();
        for raw in urls {
            let url = raw.trim().to_string();
            if url.is_empty() {
                continue;
            }
            if !(url.starts_with("http://") || url.starts_with("https://")) {
                return Err("invalid_url".into());
            }
            let id = state.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            queue.push(QueueItem {
                id,
                url,
                kind,
                status: ItemStatus::Queued,
                title: None,
                playlist_index: None,
                n_entries: None,
                percent: 0.0,
                speed: None,
                eta: None,
                downloaded: None,
                total: None,
                output_path: None,
                error: None,
                warn: None,
                created_at: now,
            });
            ids.push(id);
        }
    }
    state::emit_queue(&app, &state, true);
    state.wake.notify_one();
    Ok(ids)
}

#[tauri::command]
fn cancel_item(app: AppHandle, state: State<'_, AppState>, id: u64) {
    let is_current = state.current_id.lock().unwrap().map(|c| c == id).unwrap_or(false);
    if is_current {
        state.cancel.store(true, std::sync::atomic::Ordering::SeqCst);
        let pid = *state.current_pid.lock().unwrap();
        if let Some(pid) = pid {
            bins::kill_tree(pid);
        }
    } else {
        let mut queue = state.queue.lock().unwrap();
        if let Some(item) = queue.iter_mut().find(|i| i.id == id) {
            if item.status == ItemStatus::Queued {
                item.status = ItemStatus::Canceled;
            }
        }
        drop(queue);
        state::emit_queue(&app, &state, true);
    }
}

#[tauri::command]
fn remove_item(app: AppHandle, state: State<'_, AppState>, id: u64) {
    let is_current = state.current_id.lock().unwrap().map(|c| c == id).unwrap_or(false);
    if is_current {
        return;
    }
    {
        let mut queue = state.queue.lock().unwrap();
        queue.retain(|item| item.id != id);
    }
    state::emit_queue(&app, &state, true);
}

#[tauri::command]
fn clear_finished(app: AppHandle, state: State<'_, AppState>) {
    {
        let mut queue = state.queue.lock().unwrap();
        queue.retain(|item| {
            !matches!(
                item.status,
                ItemStatus::Done | ItemStatus::Error | ItemStatus::Canceled
            )
        });
    }
    state::emit_queue(&app, &state, true);
}

#[tauri::command]
fn get_binaries(app: AppHandle) -> bins::BinariesInfo {
    bins::binaries_info(&app)
}

#[tauri::command]
async fn install_ytdlp(app: AppHandle) -> Result<String, String> {
    bins::install_ytdlp(&app).await
}

#[tauri::command]
async fn latest_ytdlp() -> Option<String> {
    bins::latest_ytdlp_version().await
}

#[tauri::command]
async fn check_updates(app: AppHandle) -> updates::UpdateReport {
    updates::check_all(&app).await
}

#[tauri::command]
async fn install_app_update(app: AppHandle) -> Result<(), String> {
    updates::install_app_update(&app).await
}

#[tauri::command]
fn ytdlp_state(app: AppHandle) -> bins::YtdlpStatus {
    bins::ready_status(&app)
}

#[tauri::command]
fn convert_start(app: AppHandle, path: String, target: String) {
    convert::spawn_convert(app, path, target);
}

#[tauri::command]
fn convert_cancel(app: AppHandle) {
    convert::cancel(&app);
}

#[tauri::command]
fn app_info(app: AppHandle) -> AppInfo {
    AppInfo {
        version: app.package_info().version.to_string(),
        identifier: app.config().identifier.clone(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            paths::migrate_legacy(app.handle());
            let loaded = settings::load(app.handle());
            apply_window_theme(app.handle(), &loaded.theme);
            app.manage(AppState::new(loaded));

            queue::spawn_worker(app.handle().clone());
            updates::spawn_startup(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            get_queue,
            enqueue,
            cancel_item,
            remove_item,
            clear_finished,
            get_binaries,
            install_ytdlp,
            latest_ytdlp,
            check_updates,
            install_app_update,
            ytdlp_state,
            convert_start,
            convert_cancel,
            app_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
