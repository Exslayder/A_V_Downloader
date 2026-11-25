use tauri::{AppHandle, Manager};

use crate::download;
use crate::state::{emit_queue, AppState, ItemStatus};

pub fn spawn_worker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let next_id = {
                let state = app.state::<AppState>();
                let mut queue = state.queue.lock().unwrap();
                queue
                    .iter_mut()
                    .find(|item| item.status == ItemStatus::Queued)
                    .map(|item| {
                        item.status = ItemStatus::Downloading;
                        item.id
                    })
            };

            match next_id {
                Some(id) => {
                    {
                        let state = app.state::<AppState>();
                        emit_queue(&app, &state, true);
                    }
                    download::run_item(app.clone(), id).await;
                    let state = app.state::<AppState>();
                    emit_queue(&app, &state, true);
                }
                None => {
                    let state = app.state::<AppState>();
                    state.wake.notified().await;
                }
            }
        }
    });
}
