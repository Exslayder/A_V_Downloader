export type MediaKind = "mp4" | "webm" | "mp3";
export type ItemStatus =
  | "queued"
  | "downloading"
  | "processing"
  | "done"
  | "error"
  | "canceled";

export interface QueueItem {
  id: number;
  url: string;
  kind: MediaKind;
  status: ItemStatus;
  title: string | null;
  playlistIndex: number | null;
  nEntries: number | null;
  percent: number;
  speed: number | null;
  eta: number | null;
  downloaded: number | null;
  total: number | null;
  outputPath: string | null;
  error: string | null;
  warn: string | null;
  createdAt: number;
}

export type Lang = "ru" | "en";
export type Theme = "light" | "dark" | "system";

export interface Settings {
  audioPath: string;
  videoPath: string;
  language: Lang;
  autoUpdate: boolean;
  theme: Theme;
}

export interface BinariesInfo {
  ytdlpPath: string | null;
  ytdlpVersion: string | null;
  ffmpegPath: string | null;
  ffmpegVersion: string | null;
}

export interface YtdlpStatus {
  state: "ready" | "missing" | "downloading" | "error";
  version?: string | null;
  percent?: number | null;
  error?: string | null;
}

export interface ConvertProgress {
  running: boolean;
  phase: "nvenc" | "cpu" | "vp9" | "audio" | "done" | "error" | "canceled" | "";
  percent: number;
  speed?: string | null;
  file?: string | null;
  output?: string | null;
  error?: string | null;
}

export type ConvertTarget = "mp4" | "mkv" | "webm" | "mov" | "mp3" | "m4a" | "wav";

export interface AppInfo {
  version: string;
  identifier: string;
}

export interface UpdateReport {
  ytdlpCurrent: string | null;
  ytdlpLatest: string | null;
  appCurrent: string;
  appLatest: string | null;
  appError: string | null;
}

export interface AppUpdateStatus {
  phase:
    | "idle"
    | "checking"
    | "downloading"
    | "installing"
    | "available"
    | "none"
    | "error"
    | "";
  version?: string | null;
  percent?: number | null;
  error?: string | null;
}
