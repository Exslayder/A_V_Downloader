import { createContext, useContext, useMemo, type ReactNode } from "react";

import type { Lang } from "./types";

const ru = {
  "nav.downloads": "Загрузки",
  "nav.converter": "Конвертер",
  "nav.settings": "Настройки",

  "common.cancel": "Отмена",
  "common.browse": "Изменить",
  "common.retry": "Повторить",
  "common.loading": "Загрузка…",

  "downloads.title": "Загрузки",
  "downloads.subtitle": "YouTube и SoundCloud. Одна ссылка на строку.",
  "downloads.links": "Ссылки",
  "downloads.placeholder":
    "https://www.youtube.com/watch?v=…\nhttps://soundcloud.com/…",
  "downloads.mp4": "MP4",
  "downloads.webm": "WEBM",
  "downloads.mp3": "MP3 · 320",
  "downloads.download": "Скачать",
  "downloads.format": "Формат",
  "downloads.soundcloudHint": "SoundCloud — только аудио, MP3 · 320",
  "downloads.queue": "Очередь",
  "downloads.empty": "Очередь пуста",
  "downloads.emptyHint": "Вставь ссылку и выбери формат",
  "downloads.activeCount": "в работе: {count}",
  "downloads.speed": "Скорость",
  "downloads.clear": "Убрать завершённые",
  "downloads.openFolder": "Открыть папку",
  "downloads.cancel": "Отменить",
  "downloads.remove": "Удалить",
  "downloads.eta": "осталось {time}",
  "downloads.playlist": "{index} из {total}",

  "status.queued": "В очереди",
  "status.downloading": "Загрузка",
  "status.processing": "Обработка",
  "status.done": "Готово",
  "status.error": "Ошибка",
  "status.canceled": "Отменено",

  "ytdlp.missing.title": "yt-dlp не установлен",
  "ytdlp.missing.text":
    "Без него скачивание не работает. Свежая версия скачается в папку data.",
  "ytdlp.download": "Скачать",
  "ytdlp.downloading": "Скачиваем yt-dlp…",
  "ytdlp.failed": "Не удалось скачать yt-dlp",

  "err.ytdlpMissing": "yt-dlp не установлен. Скачай его в настройках.",
  "err.ffmpegMissing": "ffmpeg не найден в сборке.",
  "err.outputDir": "Не выбрана папка для сохранения.",
  "err.invalidUrl": "Ссылка должна начинаться с http:// или https://",
  "err.exitCode": "yt-dlp завершился с кодом {code}",
  "err.fileNotFound": "Файл не найден.",
  "err.alreadyMp4": "Этот файл уже MP4.",
  "err.ffmpegFailed": "ffmpeg не смог сконвертировать файл.",
  "err.sameFormat": "Файл уже в этом формате.",
  "err.noVideo": "В файле нет видеодорожки.",
  "err.noAudio": "В файле нет аудиодорожки.",
  "err.badTarget": "Неподдерживаемый формат.",
  "err.busy": "Уже выполняется другая операция.",
  "err.generic": "Не удалось выполнить операцию",
  "err.canceled": "Операция отменена.",

  "warn.title": "С ошибками: {count}",
  "warn.age": "18+ контент, нужны cookies",
  "warn.unavailable": "Видео недоступно или приватное",

  "toast.added": "Добавлено в очередь: {count}",
  "toast.ytdlpInstalled": "yt-dlp обновлён: {version}",

  "conv.title": "Конвертер",
  "conv.subtitle": "Видео и аудио в популярные форматы. Для H.264 сначала NVENC, при ошибке — CPU.",
  "conv.file": "Файл",
  "conv.format": "Формат",
  "conv.target.mp4": "MP4",
  "conv.target.mkv": "MKV",
  "conv.target.webm": "WEBM",
  "conv.target.mov": "MOV",
  "conv.target.mp3": "MP3",
  "conv.target.m4a": "M4A",
  "conv.target.wav": "WAV",
  "conv.codecH264": "H.264 · NVENC / CPU",
  "conv.codecVp9": "VP9 · Opus",
  "conv.codecMp3": "MP3 · 320 kbps",
  "conv.codecM4a": "AAC · 256 kbps",
  "conv.codecWav": "PCM · 16-bit",
  "conv.sameFormat": "Файл уже в этом формате",
  "conv.selected": "Выбранный файл",
  "conv.noFile": "Файл не выбран",
  "conv.drop": "Перетащи файл сюда или выбери вручную",
  "conv.dropActive": "Отпусти файл",
  "conv.pick": "Выбрать файл",
  "conv.start": "Конвертировать",
  "conv.cancel": "Отменить",
  "conv.phaseNvenc": "NVENC",
  "conv.phaseCpu": "CPU",
  "conv.phaseVp9": "VP9",
  "conv.phaseAudio": "Аудио",
  "conv.done": "Конвертация завершена",
  "conv.output": "Результат:",
  "conv.openFolder": "Открыть папку",

  "set.title": "Настройки",
  "set.folders": "Папки сохранения",
  "set.audio": "Музыка · MP3",
  "set.video": "Видео · MP4 / WEBM",
  "set.language": "Язык интерфейса",
  "set.appearance": "Оформление",
  "set.theme": "Тема",
  "set.themeSystem": "Системная",
  "set.themeLight": "Светлая",
  "set.themeDark": "Тёмная",
  "set.updates": "Обновления",
  "set.ytdlp": "yt-dlp",
  "set.ffmpeg": "ffmpeg",
  "set.notFound": "не найден",
  "set.about": "О программе",
  "set.version": "Версия",
  "set.check": "Проверить обновление",
  "set.checkAll": "Проверить обновления",
  "set.checking": "Проверяем…",
  "set.update": "Обновить",
  "set.updateYtdlp": "Обновить yt-dlp",
  "set.autoUpdate": "Обновлять автоматически при запуске",
  "set.autoUpdateHint":
    "Проверяет yt-dlp и AVDock при запуске, ставит обновления и перезапускает.",
  "set.appLatest": "Доступна версия {version}",
  "set.installApp": "Обновить и перезапустить",
  "set.allActual": "Всё актуально",
  "set.notChecked": "не проверялось",
  "set.newVersion": "Доступна версия {version}",
  "set.upToDate": "Установлена последняя версия",
  "set.appUpdateError": "Не удалось проверить обновления",
  "set.github": "Открыть GitHub",
  "set.dataNote":
    "Настройки, yt-dlp и временные файлы лежат в папке data рядом с программой. Если она недоступна для записи, используется профиль Windows.",

  "update.downloading": "Скачиваем обновление AVDock",
  "update.installing": "Устанавливаем обновление…",
  "update.available": "Доступно обновление AVDock {version}",
  "update.error": "Не удалось обновить AVDock",
} as const;

export type TKey = keyof typeof ru;

const en: Record<TKey, string> = {
  "nav.downloads": "Downloads",
  "nav.converter": "Converter",
  "nav.settings": "Settings",

  "common.cancel": "Cancel",
  "common.browse": "Change",
  "common.retry": "Retry",
  "common.loading": "Loading…",

  "downloads.title": "Downloads",
  "downloads.subtitle": "YouTube and SoundCloud. One link per line.",
  "downloads.links": "Links",
  "downloads.placeholder":
    "https://www.youtube.com/watch?v=…\nhttps://soundcloud.com/…",
  "downloads.mp4": "MP4",
  "downloads.webm": "WEBM",
  "downloads.mp3": "MP3 · 320",
  "downloads.download": "Download",
  "downloads.format": "Format",
  "downloads.soundcloudHint": "SoundCloud is audio only, MP3 · 320",
  "downloads.queue": "Queue",
  "downloads.empty": "Queue is empty",
  "downloads.emptyHint": "Paste a link and pick a format",
  "downloads.activeCount": "active: {count}",
  "downloads.speed": "Speed",
  "downloads.clear": "Clear finished",
  "downloads.openFolder": "Open folder",
  "downloads.cancel": "Cancel",
  "downloads.remove": "Remove",
  "downloads.eta": "{time} left",
  "downloads.playlist": "{index} of {total}",

  "status.queued": "Queued",
  "status.downloading": "Downloading",
  "status.processing": "Processing",
  "status.done": "Done",
  "status.error": "Error",
  "status.canceled": "Canceled",

  "ytdlp.missing.title": "yt-dlp is not installed",
  "ytdlp.missing.text":
    "Downloads will not work without it. The latest build goes into the data folder.",
  "ytdlp.download": "Download",
  "ytdlp.downloading": "Downloading yt-dlp…",
  "ytdlp.failed": "Failed to download yt-dlp",

  "err.ytdlpMissing": "yt-dlp is not installed. Download it in Settings.",
  "err.ffmpegMissing": "ffmpeg was not found in the bundle.",
  "err.outputDir": "No output folder selected.",
  "err.invalidUrl": "The link must start with http:// or https://",
  "err.exitCode": "yt-dlp exited with code {code}",
  "err.fileNotFound": "File not found.",
  "err.alreadyMp4": "This file is already MP4.",
  "err.ffmpegFailed": "ffmpeg could not convert the file.",
  "err.sameFormat": "The file is already in this format.",
  "err.noVideo": "The file has no video track.",
  "err.noAudio": "The file has no audio track.",
  "err.badTarget": "Unsupported format.",
  "err.busy": "Another operation is already running.",
  "err.generic": "The operation failed",
  "err.canceled": "Operation canceled.",

  "warn.title": "Finished with errors: {count}",
  "warn.age": "18+ content, cookies required",
  "warn.unavailable": "Video is unavailable or private",

  "toast.added": "Added to queue: {count}",
  "toast.ytdlpInstalled": "yt-dlp updated: {version}",

  "conv.title": "Converter",
  "conv.subtitle": "Video and audio into popular formats. H.264 goes through NVENC first, then CPU.",
  "conv.file": "File",
  "conv.format": "Format",
  "conv.target.mp4": "MP4",
  "conv.target.mkv": "MKV",
  "conv.target.webm": "WEBM",
  "conv.target.mov": "MOV",
  "conv.target.mp3": "MP3",
  "conv.target.m4a": "M4A",
  "conv.target.wav": "WAV",
  "conv.codecH264": "H.264 · NVENC / CPU",
  "conv.codecVp9": "VP9 · Opus",
  "conv.codecMp3": "MP3 · 320 kbps",
  "conv.codecM4a": "AAC · 256 kbps",
  "conv.codecWav": "PCM · 16-bit",
  "conv.sameFormat": "The file is already in this format",
  "conv.selected": "Selected file",
  "conv.noFile": "No file selected",
  "conv.drop": "Drop a file here or choose manually",
  "conv.dropActive": "Release to drop",
  "conv.pick": "Choose file",
  "conv.start": "Convert",
  "conv.cancel": "Cancel",
  "conv.phaseNvenc": "NVENC",
  "conv.phaseCpu": "CPU",
  "conv.phaseVp9": "VP9",
  "conv.phaseAudio": "Audio",
  "conv.done": "Conversion finished",
  "conv.output": "Result:",
  "conv.openFolder": "Open folder",

  "set.title": "Settings",
  "set.folders": "Save folders",
  "set.audio": "Music · MP3",
  "set.video": "Video · MP4 / WEBM",
  "set.language": "Interface language",
  "set.appearance": "Appearance",
  "set.theme": "Theme",
  "set.themeSystem": "System",
  "set.themeLight": "Light",
  "set.themeDark": "Dark",
  "set.updates": "Updates",
  "set.ytdlp": "yt-dlp",
  "set.ffmpeg": "ffmpeg",
  "set.notFound": "not found",
  "set.about": "About",
  "set.version": "Version",
  "set.check": "Check for updates",
  "set.checkAll": "Check for updates",
  "set.checking": "Checking…",
  "set.update": "Update",
  "set.updateYtdlp": "Update yt-dlp",
  "set.autoUpdate": "Update automatically on startup",
  "set.autoUpdateHint":
    "Checks yt-dlp and AVDock on startup, installs updates and restarts.",
  "set.appLatest": "Version {version} is available",
  "set.installApp": "Update and restart",
  "set.allActual": "Everything is up to date",
  "set.notChecked": "not checked",
  "set.newVersion": "Version {version} is available",
  "set.upToDate": "The latest version is installed",
  "set.appUpdateError": "Failed to check for updates",
  "set.github": "Open GitHub",
  "set.dataNote":
    "Settings, yt-dlp and temporary files live in the data folder next to the app. If it is not writable, the Windows profile is used instead.",

  "update.downloading": "Downloading AVDock update",
  "update.installing": "Installing update…",
  "update.available": "AVDock {version} is available",
  "update.error": "Failed to update AVDock",
};

const dicts: Record<Lang, Record<TKey, string>> = { ru, en };

type Vars = Record<string, string | number>;

export function translate(lang: Lang, key: TKey, vars?: Vars): string {
  const template = dicts[lang][key] ?? dicts.ru[key] ?? key;
  if (!vars) return template;
  return template.replace(/\{(\w+)\}/g, (_, name: string) =>
    name in vars ? String(vars[name]) : `{${name}}`,
  );
}

interface I18nValue {
  lang: Lang;
  t: (key: TKey, vars?: Vars) => string;
  setLang: (lang: Lang) => void;
}

const I18nContext = createContext<I18nValue>({
  lang: "ru",
  t: (key, vars) => translate("ru", key, vars),
  setLang: () => undefined,
});

export function I18nProvider({
  lang,
  setLang,
  children,
}: {
  lang: Lang;
  setLang: (lang: Lang) => void;
  children: ReactNode;
}) {
  const value = useMemo<I18nValue>(
    () => ({
      lang,
      t: (key, vars) => translate(lang, key, vars),
      setLang,
    }),
    [lang, setLang],
  );
  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  return useContext(I18nContext);
}
