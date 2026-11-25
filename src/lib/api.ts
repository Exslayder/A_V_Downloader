import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  AppInfo,
  AppUpdateStatus,
  BinariesInfo,
  ConvertProgress,
  MediaKind,
  QueueItem,
  Settings,
  UpdateReport,
  YtdlpStatus,
} from "./types";

export const api = {
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<void>("save_settings", { settings }),
  getQueue: () => invoke<QueueItem[]>("get_queue"),
  enqueue: (urls: string[], kind: MediaKind) => invoke<number[]>("enqueue", { urls, kind }),
  cancelItem: (id: number) => invoke<void>("cancel_item", { id }),
  removeItem: (id: number) => invoke<void>("remove_item", { id }),
  clearFinished: () => invoke<void>("clear_finished"),
  getBinaries: () => invoke<BinariesInfo>("get_binaries"),
  installYtdlp: () => invoke<string>("install_ytdlp"),
  latestYtdlp: () => invoke<string | null>("latest_ytdlp"),
  ytdlpState: () => invoke<YtdlpStatus>("ytdlp_state"),
  checkUpdates: () => invoke<UpdateReport>("check_updates"),
  installAppUpdate: () => invoke<void>("install_app_update"),
  convertStart: (path: string, target: string) => invoke<void>("convert_start", { path, target }),
  convertCancel: () => invoke<void>("convert_cancel"),
  appInfo: () => invoke<AppInfo>("app_info"),
};

export const events = {
  onQueue: (handler: (items: QueueItem[]) => void): Promise<UnlistenFn> =>
    listen<QueueItem[]>("queue-update", (event) => handler(event.payload)),
  onYtdlp: (handler: (status: YtdlpStatus) => void): Promise<UnlistenFn> =>
    listen<YtdlpStatus>("ytdlp-status", (event) => handler(event.payload)),
  onConvert: (handler: (progress: ConvertProgress) => void): Promise<UnlistenFn> =>
    listen<ConvertProgress>("convert-progress", (event) => handler(event.payload)),
  onConvertLog: (handler: (line: string) => void): Promise<UnlistenFn> =>
    listen<string>("convert-log", (event) => handler(event.payload)),
  onAppUpdate: (handler: (status: AppUpdateStatus) => void): Promise<UnlistenFn> =>
    listen<AppUpdateStatus>("app-update-status", (event) => handler(event.payload)),
};
