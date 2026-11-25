import { useEffect, useRef, useState } from "react";
import { animate, svg } from "animejs";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { getCurrentWebview } from "@tauri-apps/api/webview";

import { useI18n } from "../lib/i18n";
import { baseName, humanizeError } from "../lib/format";
import type { ConvertProgress, ConvertTarget } from "../lib/types";
import { Meter } from "./Meter";
import { IconAlert, IconFilm, IconFolder, IconMusic, IconSliders, IconX } from "./Icons";

const videoTargets: ConvertTarget[] = ["mp4", "mkv", "webm", "mov"];
const audioTargets: ConvertTarget[] = ["mp3", "m4a", "wav"];

const mediaExtensions = [
  "webm",
  "mkv",
  "mov",
  "avi",
  "ts",
  "flv",
  "mp4",
  "m4v",
  "mp3",
  "m4a",
  "wav",
  "flac",
  "ogg",
  "aac",
  "opus",
];

const mediaPattern = new RegExp(`\\.(${mediaExtensions.join("|")})$`, "i");

interface ConverterProps {
  convert: ConvertProgress | null;
  onStart: (file: string, target: ConvertTarget) => Promise<void>;
  onCancel: () => Promise<void>;
}

function DoneMark() {
  const pathRef = useRef<SVGPathElement>(null);

  useEffect(() => {
    if (!pathRef.current) return;
    animate(svg.createDrawable(pathRef.current), {
      draw: "0 1",
      duration: 550,
      ease: "outQuad",
    });
  }, []);

  return (
    <svg className="done-mark" viewBox="0 0 24 24" fill="none" aria-hidden>
      <path
        ref={pathRef}
        d="m4.5 12.5 4.5 4.5L19.5 6.5"
        stroke="currentColor"
        strokeWidth="1.8"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export function Converter({ convert, onStart, onCancel }: ConverterProps) {
  const { t } = useI18n();
  const [file, setFile] = useState<string | null>(null);
  const [target, setTarget] = useState<ConvertTarget>("mp4");
  const [dragging, setDragging] = useState(false);

  const running = convert?.running ?? false;
  const progress = Math.max(0, Math.min(100, convert?.percent ?? 0));

  const fileExtension = file?.split(".").pop()?.toLowerCase() ?? "";
  const sameFormat = Boolean(file && fileExtension === target);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    getCurrentWebview()
      .onDragDropEvent((event) => {
        const payload = event.payload;
        if (payload.type === "enter" || payload.type === "over") {
          setDragging(true);
          return;
        }
        setDragging(false);
        if (payload.type === "drop") {
          const media = payload.paths.find((path) => mediaPattern.test(path));
          if (media) setFile(media);
        }
      })
      .then((fn) => {
        unlisten = fn;
      })
      .catch(() => undefined);

    return () => unlisten?.();
  }, []);

  async function pick() {
    const selected = await openDialog({
      multiple: false,
      directory: false,
      filters: [
        {
          name: "Video",
          extensions: ["webm", "mkv", "mov", "avi", "ts", "flv", "mp4", "m4v"],
        },
        { name: "Audio", extensions: ["mp3", "m4a", "wav", "flac", "ogg", "aac", "opus"] },
        { name: "All files", extensions: ["*"] },
      ],
    });
    if (typeof selected === "string") {
      setFile(selected);
    }
  }

  const phaseLabel =
    convert?.phase === "nvenc"
      ? t("conv.phaseNvenc")
      : convert?.phase === "cpu"
        ? t("conv.phaseCpu")
        : convert?.phase === "vp9"
          ? t("conv.phaseVp9")
          : convert?.phase === "audio"
            ? t("conv.phaseAudio")
            : "";

  const codecHint =
    target === "webm"
      ? t("conv.codecVp9")
      : target === "mp3"
        ? t("conv.codecMp3")
        : target === "m4a"
          ? t("conv.codecM4a")
          : target === "wav"
            ? t("conv.codecWav")
            : t("conv.codecH264");

  return (
    <div className="page">
      <header className="page-header">
        <div>
          <h1>{t("conv.title")}</h1>
          <p>{t("conv.subtitle")}</p>
        </div>
      </header>

      <section className="card">
        <p className="section-label">{t("conv.file")}</p>

        <div className={`file-picker ${dragging ? "dragging" : ""}`}>
          <div className="file-icon">
            <IconSliders size={18} />
          </div>
          <div className="file-info">
            <span className="file-label">
              {dragging ? t("conv.dropActive") : file ? t("conv.selected") : t("conv.drop")}
            </span>
            {file && <span className="file-name">{baseName(file)}</span>}
          </div>
          <button type="button" className="btn" onClick={() => void pick()} disabled={running}>
            {t("conv.pick")}
          </button>
        </div>

        <div className="convert-format">
          <div className="convert-format-head">
            <p className="section-label">{t("conv.format")}</p>
            <span className="muted small">{codecHint}</span>
          </div>
          <div
            className="format-select convert-targets"
            role="radiogroup"
            aria-label={t("conv.format")}
          >
            {[...videoTargets, ...audioTargets].map((value) => (
              <button
                key={value}
                type="button"
                role="radio"
                aria-checked={target === value}
                className={`format-option ${target === value ? "selected" : ""}`}
                onClick={() => setTarget(value)}
                disabled={running}
              >
                {audioTargets.includes(value) ? (
                  <IconMusic size={15} />
                ) : (
                  <IconFilm size={15} />
                )}
                {t(`conv.target.${value}`)}
              </button>
            ))}
          </div>
        </div>

        <div className="converter-actions">
          <button
            type="button"
            className="btn btn-primary"
            disabled={!file || running || sameFormat}
            onClick={() => file && void onStart(file, target)}
          >
            <IconFilm size={16} />
            {t("conv.start")}
          </button>
          {running && (
            <button type="button" className="btn btn-danger" onClick={() => void onCancel()}>
              <IconX size={15} />
              {t("conv.cancel")}
            </button>
          )}
        </div>

        {sameFormat && !running && (
          <p className="muted small convert-note">{t("conv.sameFormat")}</p>
        )}

        {convert &&
          (running ||
            convert.phase === "done" ||
            convert.phase === "error" ||
            convert.phase === "canceled") && (
            <div className="convert-status">
              <div className="convert-status-head">
                <span className="converter-phase">
                  <span
                    className={`status-dot ${
                      convert.phase === "error"
                        ? "error"
                        : convert.phase === "done"
                          ? "done"
                          : "active"
                    }`}
                  />
                  {convert.phase === "done"
                    ? t("conv.done")
                    : convert.phase === "error"
                      ? t("status.error")
                      : convert.phase === "canceled"
                        ? t("status.canceled")
                        : phaseLabel}
                </span>
                {running && <span className="queue-metric">{Math.round(progress)}%</span>}
                {running && convert.speed && (
                  <span className="queue-metric">{convert.speed}</span>
                )}
              </div>

              {(running || convert.phase === "done") && <Meter value={progress} />}

              {convert.phase === "done" && convert.output && (
                <div className="convert-result">
                  <DoneMark />
                  <span className="file-name">{baseName(convert.output)}</span>
                  <button
                    type="button"
                    className="btn btn-ghost"
                    onClick={() => void revealIteminDirSafe(convert.output!)}
                  >
                    <IconFolder size={14} />
                    {t("conv.openFolder")}
                  </button>
                </div>
              )}

              {convert.phase === "error" && (
                <div className="queue-error">
                  <IconAlert size={14} /> {humanizeError(t, convert.error)}
                </div>
              )}
            </div>
          )}
      </section>
    </div>
  );
}

async function revealIteminDirSafe(path: string) {
  try {
    await revealItemInDir(path);
  } catch {
    /* ignore */
  }
}
