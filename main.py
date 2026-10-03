import json
import os
import shutil
import subprocess
import sys
import tkinter as tk
from tkinter import filedialog
import urllib.request
from datetime import datetime
from pathlib import Path
import yt_dlp

if getattr(sys, 'frozen', False):
    BASE_DIR = Path(sys.executable).parent
else:
    BASE_DIR = Path(__file__).parent

SETTINGS_FILE = BASE_DIR / "settings.json"
VERSION_FILE = BASE_DIR / "version.json"
DEFAULT_PATH = str(BASE_DIR)
GITHUB_API_URL = "https://api.github.com/repos/Exslayder/A_V_Downloader/releases/latest"

# ---------- AUTO UPDATE ----------
def format_github_date(iso_str):
    try:
        dt = datetime.strptime(iso_str, "%Y-%m-%dT%H:%M:%SZ")
        return dt.strftime("%d.%m.%Y %H:%M UTC")
    except Exception:
        return iso_str

def check_for_updates():
    if not getattr(sys, 'frozen', False):
        return

    print("🔄 Проверка обновлений...")
    try:
        req = urllib.request.Request(
            GITHUB_API_URL, 
            headers={"User-Agent": "Mozilla/5.0"}
        )
        with urllib.request.urlopen(req, timeout=5) as response:
            release_data = json.loads(response.read().decode())

        exe_asset = None
        for asset in release_data.get("assets", []):
            if asset.get("name") == "main.exe":
                exe_asset = asset
                break

        if not exe_asset:
            print("⚠️ Не найден main.exe в последнем релизе.")
            return

        download_url = exe_asset["browser_download_url"]
        remote_updated_at = exe_asset["updated_at"]
        formatted_date = format_github_date(remote_updated_at)

        local_updated_at = ""
        if VERSION_FILE.exists():
            try:
                with open(VERSION_FILE, "r", encoding="utf-8") as f:
                    v_data = json.load(f)
                    local_updated_at = v_data.get("updated_at", "")
            except Exception:
                pass

        if local_updated_at == remote_updated_at:
            print(f"✅ У вас установлена последняя версия! (Обновление от {formatted_date})")
            return

        print(f"🚀 Найдена новая версия от {formatted_date}! Скачивание обновления...")
        new_exe_path = BASE_DIR / "main_new.exe"
        
        urllib.request.urlretrieve(download_url, new_exe_path)

        with open(VERSION_FILE, "w", encoding="utf-8") as f:
            json.dump({"updated_at": remote_updated_at}, f, indent=4)

        current_exe = Path(sys.executable)
        updater_bat = BASE_DIR / "updater.bat"
        bat_content = f"""@echo off
timeout /t 2 /nobreak > NUL
move /y "{new_exe_path}" "{current_exe}"
start "" "{current_exe}"
del "%~f0"
"""
        with open(updater_bat, "w", encoding="utf-8") as f:
            f.write(bat_content)

        print("✨ Обновление скачано! Перезапуск программы...")
        subprocess.Popen([str(updater_bat)], shell=True)
        sys.exit(0)

    except Exception as e:
        print(f"⚠️ Ошибка при проверке обновлений: {e}")

def get_ffmpeg_path():
    if getattr(sys, 'frozen', False):
        return os.path.join(sys._MEIPASS, "ffmpeg.exe")
    
    ffmpeg_exe = shutil.which("ffmpeg")
    if ffmpeg_exe:
        return ffmpeg_exe
    
    return "ffmpeg"

# ---------- SETTINGS ----------
def load_settings():
    default_settings = {
        "audio_path": DEFAULT_PATH,
        "video_path": DEFAULT_PATH
    }

    if not os.path.exists(SETTINGS_FILE):
        save_settings(default_settings)
        return default_settings

    try:
        with open(SETTINGS_FILE, "r", encoding="utf-8") as f:
            settings = json.load(f)
            updated = False
            if "audio_path" not in settings:
                settings["audio_path"] = settings.get("download_path", DEFAULT_PATH)
                updated = True
            if "video_path" not in settings:
                settings["video_path"] = settings.get("download_path", DEFAULT_PATH)
                updated = True
            if updated:
                save_settings(settings)
            return settings
    except (json.JSONDecodeError, OSError):
        print("⚠️ settings.json повреждён, пересоздаю файл")
        save_settings(default_settings)
        return default_settings

def save_settings(settings):
    with open(SETTINGS_FILE, "w", encoding="utf-8") as f:
        json.dump(settings, f, indent=4, ensure_ascii=False)

def get_new_path_from_user(current_path_type, current_path):
    print(f"\n⚙️ ИЗМЕНЕНИЕ ПУТИ ДЛЯ: {current_path_type.upper()}")
    print(f"Текущий путь: {current_path}")
    new_path = input("Введи новый путь (или Enter чтобы оставить как есть): ").strip()
    
    if not new_path:
        return current_path

    path = Path(new_path).expanduser()
    if path.exists() and path.is_dir():
        return str(path.resolve())
    else:
        print("❌ Путь не существует или это не папка! Изменения не сохранены.")
        return current_path

def settings_menu(settings):
    while True:
        print("\n⚙️ НАСТРОЙКИ ПУТЕЙ")
        print(f"🎵 Для АУДИО: {settings['audio_path']}")
        print(f"🎬 Для ВИДЕО: {settings['video_path']}")
        print("1) Изменить путь для АУДИО")
        print("2) Изменить путь для ВИДЕО")
        print("\n0) ⬅ Назад")

        choice = input("Номер: ").strip()
        if choice == "0": break
        elif choice == "1":
            settings["audio_path"] = get_new_path_from_user("аудио", settings["audio_path"])
            save_settings(settings)
        elif choice == "2":
            settings["video_path"] = get_new_path_from_user("видео", settings["video_path"])
            save_settings(settings)

# ---------- CONVERT ----------
def get_video_duration(input_file, ffmpeg_bin):
    cmd = [
        ffmpeg_bin,
        "-i", str(input_file)
    ]
    process = subprocess.Popen(cmd, stderr=subprocess.PIPE, universal_newlines=True, encoding="utf-8", errors="ignore")
    _, stderr = process.communicate()
    for line in stderr.splitlines():
        if "Duration:" in line:
            try:
                time_str = line.split("Duration:")[1].split(",")[0].strip()
                h, m, s = time_str.split(":")
                return float(h) * 3600 + float(m) * 60 + float(s)
            except Exception:
                pass
    return 0.0

def convert_webm_to_mp4():
    print("\n📂 Открываю окно выбора файла...")

    file_path = ""
    try:
        root = tk.Tk()
        root.withdraw()
        root.attributes("-topmost", True)
        
        file_path = filedialog.askopenfilename(
            title="Выберите WEBM файл для конвертации",
            filetypes=[("WEBM видео", "*.webm"), ("Все файлы", "*.*")]
        )
        root.destroy()
    except Exception:
        pass

    if not file_path:
        print("💡 Вы можете перетащить файл мышью в это окно консоли.")
        file_path = input("Вставь путь к файлу .webm (или Enter для отмены): ").strip().strip('"')

    if not file_path:
        print("❌ Файл не выбран.")
        return

    input_file = Path(file_path)

    if not input_file.exists() or not input_file.is_file():
        print("❌ Ошибка: Указанный файл не найден!")
        return

    output_file = input_file.with_suffix(".mp4")
    ffmpeg_bin = get_ffmpeg_path()

    total_duration = get_video_duration(input_file, ffmpeg_bin)

    cmd = [
        ffmpeg_bin,
        "-y",
        "-i", str(input_file),
        "-c:v", "libx264",
        "-crf", "18",
        "-c:a", "aac",
        "-progress", "pipe:1",
        "-nostats",
        str(output_file)
    ]

    print(f"\n🎬 Начинаю конвертацию файла: {input_file.name}...")

    try:
        process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            universal_newlines=True,
            encoding="utf-8",
            errors="ignore"
        )

        out_time_ms = 0
        speed = "1x"

        for line in process.stdout:
            line = line.strip()
            if line.startswith("out_time_ms="):
                try:
                    val = line.split("=")[1]
                    out_time_ms = int(val)
                except ValueError:
                    pass
            elif line.startswith("speed="):
                speed = line.split("=")[1].strip()
            elif line.startswith("progress="):
                current_sec = out_time_ms / 1_000_000.0
                if total_duration > 0:
                    percent = min(100.0, (current_sec / total_duration) * 100)
                    print(f"\r⚙️\tКонвертация | ⬇ {percent:.1f}% | Скорость: {speed}   ", end="", flush=True)
                else:
                    m, s = divmod(int(current_sec), 60)
                    print(f"\r⚙️\tОбработано: {m:02d}:{s:02d} | Скорость: {speed}   ", end="", flush=True)

        process.wait()

        if process.returncode == 0:
            print(f"\n\n✨\tКОНВЕРТАЦИЯ ЗАВЕРШЕНА")
            print(f"🎉\tФайл успешно сохранён: {output_file}")
        else:
            print("\n❌ Ошибка во время конвертации через FFmpeg.")

    except Exception as e:
        print(f"\n⚠️ Произошла ошибка: {e}")

# ---------- DOWNLOAD ----------
def choose_options(choice: str):
    if choice == "1":
        return {
            "format": "bv*[ext=mp4]+ba[ext=m4a]/bv*+ba/best",
            "merge_output_format": "mp4",
        }

    elif choice == "2":
        return {
            "format": "bv*[ext=webm]+ba[ext=webm]/bv*+ba/best",
            "merge_output_format": "webm",
        }
        
    elif choice == "3":
        return {
            "format": "bestaudio/best",
            "writethumbnail": True,
            "postprocessors": [
                {
                    "key": "FFmpegExtractAudio",
                    "preferredcodec": "mp3",
                    "preferredquality": "320",
                },
                {
                    "key": "FFmpegThumbnailsConvertor",
                    "format": "jpg",
                },
                {
                    "key": "EmbedThumbnail",
                },
                {
                    "key": "FFmpegMetadata",
                    "add_metadata": True,
                },
            ],
            "postprocessor_args": {
                "ffmpeg": [
                    "-id3v2_version", "3",
                    "-metadata:s:v", "title=\"Album cover\"",
                    "-metadata:s:v", "comment=\"Cover (Front)\""
                ]
            },
        }
    return None

def progress_hook(d):
    if d["status"] == "downloading":
        info = d.get('info_dict', {})
        p_index = info.get('playlist_index')
        n_entries = info.get('n_entries')
        if p_index is None:
            p_index = 1
            n_entries = 1
        percent = d.get("_percent_str", "").strip()
        speed = d.get("_speed_str", "").strip()
        print(f"\r📦\tФайл [{p_index}/{n_entries}] | ⬇ {percent} | {speed}          ", end="", flush=True)

class MyLogger:
    def __init__(self):
        self.error_count = 0
        self.reasons = set()

    def debug(self, msg): pass
    def warning(self, msg): pass

    def error(self, msg):
        if "Sign in to confirm your age" in msg:
            self.error_count += 1
            self.reasons.add("18+ content. Need cookies")
        elif "is unavailable" in msg or "private" in msg:
            self.error_count += 1
            self.reasons.add("Video is unavailable or private")
        elif "ERROR:" in msg:
            self.error_count += 1
            clean_err = msg.split(':')[ -1 ].strip().split('.')[0]
            self.reasons.add(clean_err)

def download_flow(choice, settings):
    url = input("\nВставь ссылку: ").strip()
    opts = choose_options(choice)
    if not opts: return
    
    download_path = settings["audio_path"] if choice == "3" else settings["video_path"]
    outtmpl = "%(title)s - %(uploader)s.%(ext)s" if choice == "3" else "%(title)s.%(ext)s"

    logger_instance = MyLogger()
    
    ydl_opts = {
        "outtmpl": os.path.join(download_path, outtmpl),
        "noplaylist": False,
        "progress_hooks": [progress_hook],
        "ffmpeg_location": get_ffmpeg_path(),
        "quiet": True,
        "no_warnings": True,
        "noprogress": True,
        "ignoreerrors": True, 
        "logger": logger_instance,
        "writethumbnail": opts.get("writethumbnail", False),
        "overwrites": True,
        "updatetime": False,
        **opts
    }

    print("\n🚀 Начинаю загрузку...")

    with yt_dlp.YoutubeDL(ydl_opts) as ydl:
        ydl.download([url])
    if choice == "3":
        image_extensions = {".jpg", ".jpeg", ".webp", ".png"}
        for file in os.listdir(download_path):
            file_path = os.path.join(download_path, file)
            name, ext = os.path.splitext(file)
            if ext.lower() in image_extensions:
                if not os.path.exists(os.path.join(download_path, name + ".mp3")):
                    try:
                        os.remove(file_path)
                    except:
                        pass

    print(f"\n\n✨\tСКАЧИВАНИЕ ЗАВЕРШЕНО")
    if logger_instance.error_count > 0:
        media_type = "аудио" if choice == "3" else "видео"
        reasons = ", ".join(logger_instance.reasons)
        print(f"⚠️\tНе удалось скачать {logger_instance.error_count} {media_type} по причине: {reasons}")
    else:
        print(f"🎉\tВсе файлы успешно сохранены!")

# ---------- MENU ----------
def main():
    check_for_updates()
    settings = load_settings()
    while True:
        print("\n📥 Youtube and SoundCloud DOWNLOADER")
        print("1) 🎥\tВидео MP4 - максимально доступное качество")
        print("2) 🎬\tВидео WEBM - максимально доступное качество")
        print("3) 🔊\tАудио MP3 - 320kbps")
        print("4) 🔄\tКонвертация из WEBM в MP4")
        print("5) 🔧\tНастройки")
        print("0) ❌\tВыход")

        choice = input("Номер: ").strip()
        if choice == "0": break
        elif choice == "4": convert_webm_to_mp4()
        elif choice == "5": settings_menu(settings)
        elif choice in {"1", "2", "3"}: download_flow(choice, settings)

if __name__ == "__main__":
    main()