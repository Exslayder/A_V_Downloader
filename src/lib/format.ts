import type { TKey } from "./i18n";

export function formatBytes(bytes: number | null | undefined): string {
  if (bytes == null || !Number.isFinite(bytes) || bytes <= 0) return "—";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let index = 0;
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024;
    index += 1;
  }
  const digits = value >= 100 || index === 0 ? 0 : 1;
  return `${value.toFixed(digits)} ${units[index]}`;
}

export function formatSpeed(bytesPerSecond: number | null | undefined): string | null {
  if (bytesPerSecond == null || !Number.isFinite(bytesPerSecond) || bytesPerSecond <= 0) {
    return null;
  }
  return `${formatBytes(bytesPerSecond)}/s`;
}

export function formatEta(seconds: number | null | undefined): string | null {
  if (seconds == null || !Number.isFinite(seconds) || seconds < 0) return null;
  const total = Math.round(seconds);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  if (h > 0) return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  return `${m}:${String(s).padStart(2, "0")}`;
}

export function hostOf(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, "");
  } catch {
    return url;
  }
}

export function baseName(path: string): string {
  const normalized = path.replace(/\\/g, "/");
  const parts = normalized.split("/");
  return parts[parts.length - 1] || path;
}

export function parentDir(path: string): string {
  const normalized = path.replace(/\\/g, "/");
  const index = normalized.lastIndexOf("/");
  return index > 0 ? normalized.slice(0, index) : path;
}

export function humanizeError(
  t: (key: TKey, vars?: Record<string, string | number>) => string,
  code: string | null | undefined,
): string {
  if (!code) return "";
  const [head, ...rest] = code.split(":");
  const tail = rest.join(":").trim();

  switch (head) {
    case "ytdlp_missing":
      return t("err.ytdlpMissing");
    case "ffmpeg_missing":
      return t("err.ffmpegMissing");
    case "output_dir_empty":
      return t("err.outputDir");
    case "invalid_url":
      return t("err.invalidUrl");
    case "exit_code":
      return t("err.exitCode", { code: tail || "?" });
    case "file_not_found":
      return t("err.fileNotFound");
    case "already_mp4":
      return t("err.alreadyMp4");
    case "ffmpeg_failed":
      return t("err.ffmpegFailed");
    case "same_format":
      return t("err.sameFormat");
    case "no_video":
      return t("err.noVideo");
    case "no_audio":
      return t("err.noAudio");
    case "bad_target":
      return t("err.badTarget");
    case "busy":
      return t("err.busy");
    case "canceled":
      return t("err.canceled");
    default:
      return code.startsWith("raw:") ? code.slice(4) : code;
  }
}

export function parseWarn(
  t: (key: TKey, vars?: Record<string, string | number>) => string,
  warn: string | null,
): { title: string; reasons: string[] } | null {
  if (!warn) return null;
  const [countRaw, reasonsRaw = ""] = warn.split("|");
  const count = Number.parseInt(countRaw, 10);
  const reasons = reasonsRaw
    .split(",")
    .map((reason) => reason.trim())
    .filter(Boolean)
    .map((reason) => {
      if (reason === "age_restricted") return t("warn.age");
      if (reason === "unavailable") return t("warn.unavailable");
      if (reason.startsWith("raw:")) return reason.slice(4);
      return reason;
    });
  return {
    title: t("warn.title", { count: Number.isFinite(count) ? count : reasons.length }),
    reasons,
  };
}
