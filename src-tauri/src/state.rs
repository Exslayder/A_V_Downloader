use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Mutex;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Notify;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub audio_path: String,
    pub video_path: String,
    pub language: String,
    #[serde(default = "default_true")]
    pub auto_update: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_true() -> bool {
    true
}

fn default_theme() -> String {
    "system".into()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            audio_path: String::new(),
            video_path: String::new(),
            language: "ru".into(),
            auto_update: true,
            theme: default_theme(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Mp4,
    Webm,
    Mp3,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ItemStatus {
    Queued,
    Downloading,
    Processing,
    Done,
    Error,
    Canceled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueItem {
    pub id: u64,
    pub url: String,
    pub kind: MediaKind,
    pub status: ItemStatus,
    pub title: Option<String>,
    pub playlist_index: Option<u64>,
    pub n_entries: Option<u64>,
    pub percent: f64,
    pub speed: Option<f64>,
    pub eta: Option<u64>,
    pub downloaded: Option<u64>,
    pub total: Option<u64>,
    pub output_path: Option<String>,
    pub error: Option<String>,
    pub warn: Option<String>,
    pub created_at: u64,
}

pub struct AppState {
    pub settings: Mutex<Settings>,
    pub queue: Mutex<Vec<QueueItem>>,
    pub next_id: AtomicU64,
    pub wake: Notify,
    pub cancel: AtomicBool,
    pub current_id: Mutex<Option<u64>>,
    pub current_pid: Mutex<Option<u32>>,
    pub last_emit: Mutex<Instant>,
    pub ytdlp_busy: AtomicBool,
    pub convert_busy: AtomicBool,
    pub convert_pid: Mutex<Option<u32>>,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings: Mutex::new(settings),
            queue: Mutex::new(Vec::new()),
            next_id: AtomicU64::new(1),
            wake: Notify::new(),
            cancel: AtomicBool::new(false),
            current_id: Mutex::new(None),
            current_pid: Mutex::new(None),
            last_emit: Mutex::new(Instant::now() - std::time::Duration::from_secs(5)),
            ytdlp_busy: AtomicBool::new(false),
            convert_busy: AtomicBool::new(false),
            convert_pid: Mutex::new(None),
        }
    }
}

pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn emit_queue(app: &AppHandle, state: &AppState, force: bool) {
    {
        let mut last = state.last_emit.lock().unwrap();
        if !force && last.elapsed().as_millis() < 120 {
            return;
        }
        *last = Instant::now();
    }
    let items = state.queue.lock().unwrap().clone();
    let _ = app.emit("queue-update", items);
}

/// Applies a change to a single queue item and emits the queue if it existed.
pub fn update_item<F: FnOnce(&mut QueueItem)>(app: &AppHandle, id: u64, force: bool, f: F) {
    let state = app.state::<AppState>();
    {
        let mut queue = state.queue.lock().unwrap();
        if let Some(item) = queue.iter_mut().find(|i| i.id == id) {
            f(item);
        } else {
            return;
        }
    }
    emit_queue(app, &state, force);
}
