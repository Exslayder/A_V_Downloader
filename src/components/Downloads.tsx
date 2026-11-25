import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { revealItemInDir } from "@tauri-apps/plugin-opener";

import { useI18n } from "../lib/i18n";
import {
  baseName,
  formatBytes,
  formatEta,
  formatSpeed,
  hostOf,
  humanizeError,
  parseWarn,
} from "../lib/format";
import type { MediaKind, QueueItem, YtdlpStatus } from "../lib/types";
import { SpeedChart } from "./SpeedChart";
import { Meter } from "./Meter";
import {
  IconAlert,
  IconDownload,
  IconFilm,
  IconFolder,
  IconMusic,
  IconTrash,
  IconX,
} from "./Icons";

type Notify = (message: string, kind?: "info" | "error" | "success") => void;

interface DownloadsProps {
  queue: QueueItem[];
  ytdlp: YtdlpStatus;
  onEnqueue: (urls: string[], kind: MediaKind) => Promise<void>;
  onCancel: (id: number) => Promise<void>;
  onRemove: (id: number) => Promise<void>;
  onClearFinished: () => Promise<void>;
  onInstallYtdlp: () => Promise<void>;
  notify: Notify;
}

const kindLabel: Record<MediaKind, string> = { mp4: "MP4", webm: "WEBM", mp3: "MP3" };

function isSoundCloud(url: string): boolean {
  try {
    const host = new URL(url).hostname.toLowerCase();
    return (
      host === "soundcloud.com" ||
      host.endsWith(".soundcloud.com") ||
      host === "snd.sc" ||
      host === "on.soundcloud.com"
    );
  } catch {
    return false;
  }
}

export function Downloads({
  queue,
  ytdlp,
  onEnqueue,
  onCancel,
  onRemove,
  onClearFinished,
  onInstallYtdlp,
  notify,
}: DownloadsProps) {
  const { t } = useI18n();
  const [text, setText] = useState("");
  const [adding, setAdding] = useState(false);
  const [kind, setKind] = useState<MediaKind>("mp4");
  const [samples, setSamples] = useState<Record<number, number[]>>({});
  const inputRef = useRef<HTMLTextAreaElement>(null);

  const active = queue.filter(
    (item) => item.status === "queued" || item.status === "downloading" || item.status === "processing",
  ).length;
  const finished = queue.length - active;

  const urls = text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);

  const soundcloudOnly = urls.length > 0 && urls.every(isSoundCloud);
  const previousSoundcloud = useRef(false);

  useEffect(() => {
    if (soundcloudOnly && !previousSoundcloud.current) {
      setKind("mp3");
    } else if (!soundcloudOnly && previousSoundcloud.current) {
      setKind("mp4");
    }
    previousSoundcloud.current = soundcloudOnly;
  }, [soundcloudOnly]);

  useEffect(() => {
    setSamples((previous) => {
      const next: Record<number, number[]> = {};
      let changed = false;

      for (const item of queue) {
        const live = item.status === "downloading" || item.status === "processing";
        if (!live) continue;
        if (item.speed && item.speed > 0) {
          next[item.id] = [...(previous[item.id] ?? []), item.speed].slice(-120);
          changed = true;
        } else if (previous[item.id]) {
          next[item.id] = previous[item.id];
        }
      }

      return changed ? next : previous;
    });
  }, [queue]);

  async function add() {
    if (urls.length === 0) {
      inputRef.current?.focus();
      return;
    }
    if (adding) return;
    setAdding(true);
    try {
      await onEnqueue(urls, kind);
      setText("");
    } finally {
      setAdding(false);
    }
  }

  async function openFolder(item: QueueItem) {
    try {
      if (item.outputPath) {
        await revealItemInDir(item.outputPath);
      } else {
        notify(t("downloads.openFolder"), "error");
      }
    } catch (error) {
      notify(String(error), "error");
    }
  }

  return (
    <div className="page">
      <header className="page-header">
        <div>
          <h1>{t("downloads.title")}</h1>
          <p>{t("downloads.subtitle")}</p>
        </div>
        {active > 0 && (
          <div className="page-meta">
            <span>{t("downloads.activeCount", { count: active })}</span>
          </div>
        )}
      </header>

      <section className="card input-card">
        <p className="section-label">{t("downloads.links")}</p>
        <textarea
          ref={inputRef}
          className="url-input"
          rows={3}
          spellCheck={false}
          placeholder={t("downloads.placeholder")}
          value={text}
          onChange={(event) => setText(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
              event.preventDefault();
              void add();
            }
          }}
        />
        <div className="download-row">
          <div className="format-select" role="radiogroup" aria-label={t("downloads.format")}>
            {(["mp4", "webm", "mp3"] as MediaKind[]).map((value) => (
              <button
                key={value}
                type="button"
                role="radio"
                aria-checked={kind === value}
                className={`format-option ${kind === value ? "selected" : ""}`}
                onClick={() => setKind(value)}
                disabled={adding || (soundcloudOnly && value !== "mp3")}
                title={
                  soundcloudOnly && value !== "mp3" ? t("downloads.soundcloudHint") : undefined
                }
              >
                {value === "mp3" ? <IconMusic size={16} /> : <IconFilm size={16} />}
                {t(`downloads.${value}`)}
              </button>
            ))}
          </div>
          <button
            type="button"
            className="btn btn-primary download-btn"
            disabled={adding}
            onClick={() => void add()}
          >
            <IconDownload size={16} />
            {t("downloads.download")}
          </button>
        </div>
        {soundcloudOnly && (
          <p className="download-note muted small">{t("downloads.soundcloudHint")}</p>
        )}
      </section>

      {ytdlp.state !== "ready" && (
        <section className={`banner ${ytdlp.state === "error" ? "banner-error" : ""}`}>
          <div className="banner-icon">
            <IconAlert size={18} />
          </div>
          <div className="banner-body">
            <strong>
              {ytdlp.state === "downloading" ? t("ytdlp.downloading") : t("ytdlp.missing.title")}
            </strong>
            <span>{ytdlp.state === "error" ? t("ytdlp.failed") : t("ytdlp.missing.text")}</span>
            {ytdlp.state === "downloading" && <Meter value={ytdlp.percent ?? 0} />}
            {ytdlp.error && <span className="mono">{ytdlp.error}</span>}
          </div>
          {ytdlp.state !== "downloading" && (
            <button type="button" className="btn btn-primary" onClick={() => void onInstallYtdlp()}>
              {ytdlp.state === "error" ? t("common.retry") : t("ytdlp.download")}
            </button>
          )}
        </section>
      )}

      <section className="queue">
        <div className="queue-header">
          <h2>
            {t("downloads.queue")}
            <span className="count">{queue.length}</span>
          </h2>
          {finished > 0 && (
            <button type="button" className="btn btn-ghost" onClick={() => void onClearFinished()}>
              <IconTrash size={14} />
              {t("downloads.clear")}
            </button>
          )}
        </div>

        {queue.length === 0 ? (
          <div className="empty">
            <div className="empty-icon">
              <IconDownload size={17} />
            </div>
            <div className="empty-text">
              <strong>{t("downloads.empty")}</strong>
              <span>{t("downloads.emptyHint")}</span>
            </div>
          </div>
        ) : (
          <ul className="queue-list">
            <AnimatePresence initial={false}>
              {queue.map((item) => (
                <QueueCard
                  key={item.id}
                  item={item}
                  samples={samples[item.id] ?? []}
                  onCancel={onCancel}
                  onRemove={onRemove}
                  onOpenFolder={openFolder}
                />
              ))}
            </AnimatePresence>
          </ul>
        )}
      </section>
    </div>
  );
}

function QueueCard({
  item,
  samples,
  onCancel,
  onRemove,
  onOpenFolder,
}: {
  item: QueueItem;
  samples: number[];
  onCancel: (id: number) => Promise<void>;
  onRemove: (id: number) => Promise<void>;
  onOpenFolder: (item: QueueItem) => Promise<void>;
}) {
  const { t } = useI18n();
  const warn = parseWarn(t, item.warn);
  const running = item.status === "downloading" || item.status === "processing";
  const progress = Math.max(0, Math.min(100, item.percent || 0));
  const speed = formatSpeed(item.speed);
  const eta = formatEta(item.eta);
  const dotClass =
    item.status === "done"
      ? "done"
      : item.status === "error"
        ? "error"
        : running
          ? "active"
          : "";

  return (
    <motion.li
      layout
      initial={{ opacity: 0, y: 4 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.18, ease: [0.2, 0, 0, 1] }}
      className={`queue-item status-${item.status}`}
    >
      <div className="kind-badge">
        {item.kind === "mp3" ? <IconMusic size={17} /> : <IconFilm size={17} />}
      </div>

      <div className="queue-main">
        <div className="queue-title-row">
          <span className="queue-title">{item.title || hostOf(item.url)}</span>
          {item.playlistIndex != null && item.nEntries != null && item.nEntries > 1 && (
            <span className="playlist-chip">
              {t("downloads.playlist", {
                index: item.playlistIndex,
                total: item.nEntries,
              })}
            </span>
          )}
        </div>

        <div className="queue-sub">
          <span className="queue-kind">{kindLabel[item.kind]}</span>
          <span className="status-label">
            <span className={`status-dot ${dotClass}`} />
            {t(`status.${item.status}`)}
          </span>
          <span className="queue-url" title={item.url}>
            {item.status === "done" && item.outputPath
              ? baseName(item.outputPath)
              : hostOf(item.url)}
          </span>
          {running && (
            <>
              <span className="queue-metric">{Math.round(progress)}%</span>
              {speed && <span className="queue-metric">{speed}</span>}
              {eta && (
                <span className="queue-metric">{t("downloads.eta", { time: eta })}</span>
              )}
              {item.total != null && (
                <span className="queue-metric">
                  {formatBytes(item.downloaded)} / {formatBytes(item.total)}
                </span>
              )}
            </>
          )}
        </div>

        {running && (
          <>
            <Meter value={progress} processing={item.status === "processing"} />
            <SpeedChart samples={samples} />
          </>
        )}

        {item.status === "error" && item.error && (
          <div className="queue-error">{humanizeError(t, item.error)}</div>
        )}

        {warn && (
          <div className="queue-warn">
            <strong>{warn.title}</strong>
            {warn.reasons.length > 0 && <span>{warn.reasons.join(" В· ")}</span>}
          </div>
        )}
      </div>

      <div className="queue-actions">
        {item.status === "done" && item.outputPath && (
          <button
            type="button"
            className="icon-btn"
            title={t("downloads.openFolder")}
            onClick={() => void onOpenFolder(item)}
          >
            <IconFolder size={15} />
          </button>
        )}
        {(running || item.status === "queued") && (
          <button
            type="button"
            className="icon-btn danger"
            title={t("downloads.cancel")}
            onClick={() => void onCancel(item.id)}
          >
            <IconX size={15} />
          </button>
        )}
        {!running && item.status !== "queued" && (
          <button
            type="button"
            className="icon-btn"
            title={t("downloads.remove")}
            onClick={() => void onRemove(item.id)}
          >
            <IconTrash size={15} />
          </button>
        )}
      </div>
    </motion.li>
  );
}
