import { useCallback, useEffect, useRef, useState } from "react";
import { AnimatePresence, MotionConfig, motion } from "motion/react";
import type { UnlistenFn } from "@tauri-apps/api/event";

import { api, events } from "./lib/api";
import { humanizeError } from "./lib/format";
import { I18nProvider, translate } from "./lib/i18n";
import type {
  AppInfo,
  AppUpdateStatus,
  BinariesInfo,
  ConvertProgress,
  ConvertTarget,
  Lang,
  MediaKind,
  QueueItem,
  Settings,
  YtdlpStatus,
} from "./lib/types";
import { Sidebar, type Page } from "./components/Sidebar";
import { Downloads } from "./components/Downloads";
import { Converter } from "./components/Converter";
import { SettingsPage } from "./components/SettingsPage";
import { Meter } from "./components/Meter";
import { IconAlert, IconCheck, IconLogo } from "./components/Icons";

type ToastKind = "info" | "error" | "success";
interface Toast {
  id: number;
  message: string;
  kind: ToastKind;
}

export default function App() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [page, setPage] = useState<Page>("downloads");
  const [queue, setQueue] = useState<QueueItem[]>([]);
  const [ytdlp, setYtdlp] = useState<YtdlpStatus>({ state: "missing" });
  const [binaries, setBinaries] = useState<BinariesInfo | null>(null);
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [convert, setConvert] = useState<ConvertProgress | null>(null);
  const [appUpdate, setAppUpdate] = useState<AppUpdateStatus | null>(null);
  const [toasts, setToasts] = useState<Toast[]>([]);
  const toastId = useRef(0);

  const notify = useCallback((message: string, kind: ToastKind = "info") => {
    const id = ++toastId.current;
    setToasts((current) => [...current, { id, message, kind }]);
    window.setTimeout(() => {
      setToasts((current) => current.filter((toast) => toast.id !== id));
    }, 4200);
  }, []);

  const refreshBinaries = useCallback(async () => {
    try {
      setBinaries(await api.getBinaries());
    } catch {
      /* ignore */
    }
  }, []);

  useEffect(() => {
    let disposed = false;
    const unlisteners: UnlistenFn[] = [];

    (async () => {
      const [initialSettings, initialQueue, initialYtdlp, initialBinaries, initialAppInfo] =
        await Promise.all([
          api.getSettings(),
          api.getQueue(),
          api.ytdlpState(),
          api.getBinaries(),
          api.appInfo(),
        ]);
      if (disposed) return;
      setSettings(initialSettings);
      setQueue(initialQueue);
      setYtdlp(initialYtdlp);
      setBinaries(initialBinaries);
      setAppInfo(initialAppInfo);
      document.documentElement.lang = initialSettings.language;

      const listeners = await Promise.all([
        events.onQueue((items) => setQueue(items)),
        events.onYtdlp((status) => {
          setYtdlp(status);
          if (status.state === "ready") void refreshBinaries();
        }),
        events.onConvert((progress) => setConvert(progress)),
        events.onAppUpdate((status) => setAppUpdate(status)),
      ]);
      if (disposed) {
        listeners.forEach((unlisten) => unlisten());
      } else {
        unlisteners.push(...listeners);
      }
    })().catch((error) => {
      console.error(error);
    });

    return () => {
      disposed = true;
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [refreshBinaries]);

  useEffect(() => {
    if (settings) document.documentElement.lang = settings.language;
  }, [settings]);

  useEffect(() => {
    if (!settings) return;
    const media = window.matchMedia("(prefers-color-scheme: dark)");

    const apply = () => {
      const resolved =
        settings.theme === "system" ? (media.matches ? "dark" : "light") : settings.theme;
      document.documentElement.dataset.theme = resolved;
      document.documentElement.style.colorScheme = resolved;
    };

    apply();
    if (settings.theme === "system") {
      media.addEventListener("change", apply);
      return () => media.removeEventListener("change", apply);
    }
  }, [settings?.theme, settings]);

  const lang: Lang = settings?.language ?? "ru";
  const t = useCallback(
    (key: Parameters<typeof translate>[1], vars?: Parameters<typeof translate>[2]) =>
      translate(lang, key, vars),
    [lang],
  );

  async function persist(next: Settings) {
    setSettings(next);
    try {
      await api.saveSettings(next);
    } catch (error) {
      notify(String(error), "error");
    }
  }

  async function handleEnqueue(urls: string[], kind: MediaKind) {
    try {
      const ids = await api.enqueue(urls, kind);
      notify(t("toast.added", { count: ids.length }), "success");
    } catch (error) {
      notify(humanizeError(t, String(error)) || t("err.generic"), "error");
    }
  }

  async function handleCancel(id: number) {
    try {
      await api.cancelItem(id);
    } catch (error) {
      notify(String(error), "error");
    }
  }

  async function handleRemove(id: number) {
    try {
      await api.removeItem(id);
    } catch (error) {
      notify(String(error), "error");
    }
  }

  async function handleClearFinished() {
    try {
      await api.clearFinished();
    } catch (error) {
      notify(String(error), "error");
    }
  }

  async function handleInstallYtdlp() {
    try {
      const version = await api.installYtdlp();
      notify(t("toast.ytdlpInstalled", { version }), "success");
      await refreshBinaries();
    } catch (error) {
      notify(`${t("ytdlp.failed")}: ${error}`, "error");
    }
  }

  async function handleConvertStart(file: string, target: ConvertTarget) {
    const phase =
      target === "webm"
        ? "vp9"
        : target === "mp3" || target === "m4a" || target === "wav"
          ? "audio"
          : "nvenc";
    setConvert({ running: true, phase, percent: 0, file });
    try {
      await api.convertStart(file, target);
    } catch (error) {
      notify(humanizeError(t, String(error)) || t("err.generic"), "error");
    }
  }

  async function handleConvertCancel() {
    try {
      await api.convertCancel();
    } catch (error) {
      notify(String(error), "error");
    }
  }

  if (!settings) {
    return (
      <div className="splash">
        <span className="brand-mark">
          <IconLogo size={22} />
        </span>
        <span>AVDock</span>
      </div>
    );
  }

  const activeCount = queue.filter(
    (item) =>
      item.status === "queued" || item.status === "downloading" || item.status === "processing",
  ).length;

  return (
    <I18nProvider lang={settings.language} setLang={(next) => void persist({ ...settings, language: next })}>
      <MotionConfig reducedMotion="user">
      <div className="app">
        <Sidebar
          page={page}
          onNavigate={setPage}
          activeCount={activeCount}
        />
        <main className="content">
          {appUpdate &&
            (appUpdate.phase === "downloading" || appUpdate.phase === "installing") && (
              <div className="update-bar">
                <span className="update-bar-text">
                  {appUpdate.phase === "downloading"
                    ? t("update.downloading")
                    : t("update.installing")}
                  {appUpdate.version ? ` В· v${appUpdate.version}` : ""}
                </span>
                <Meter value={appUpdate.percent ?? 0} />
              </div>
            )}
          <AnimatePresence mode="wait" initial={false}>
            <motion.div
              key={page}
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.18, ease: [0.2, 0, 0, 1] }}
            >
              {page === "downloads" && (
                <Downloads
                  queue={queue}
                  ytdlp={ytdlp}
                  onEnqueue={handleEnqueue}
                  onCancel={handleCancel}
                  onRemove={handleRemove}
                  onClearFinished={handleClearFinished}
                  onInstallYtdlp={handleInstallYtdlp}
                  notify={notify}
                />
              )}
              {page === "converter" && (
                <Converter
                  convert={convert}
                  onStart={handleConvertStart}
                  onCancel={handleConvertCancel}
                />
              )}
              {page === "settings" && (
                <SettingsPage
                  settings={settings}
                  binaries={binaries}
                  ytdlp={ytdlp}
                  appInfo={appInfo}
                  onSave={persist}
                  onRefresh={refreshBinaries}
                  notify={notify}
                />
              )}
            </motion.div>
          </AnimatePresence>
        </main>

        <div className="toasts">
          <AnimatePresence initial={false}>
            {toasts.map((toast) => (
              <motion.div
                key={toast.id}
                className={`toast toast-${toast.kind}`}
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.18, ease: [0.2, 0, 0, 1] }}
              >
                {toast.kind === "error" ? <IconAlert size={16} /> : <IconCheck size={16} />}
                <span>{toast.message}</span>
              </motion.div>
            ))}
          </AnimatePresence>
        </div>
      </div>
      </MotionConfig>
    </I18nProvider>
  );
}
