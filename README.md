# AVDock

Настольное приложение для Windows: скачивает видео и музыку с YouTube и SoundCloud и конвертирует медиа в популярные форматы.

AVDock — новая версия проекта `A_V_Downloader`: тот же движок загрузки (yt-dlp) и те же режимы, но вместо консольного меню интерфейс на Tauri 2 + React с Rust-бэкендом.

## Возможности

- **Скачивание** — YouTube и SoundCloud, видео MP4/WEBM и аудио MP3 320 kbps.
- **SoundCloud → MP3** — при вставке ссылки SoundCloud формат автоматически переключается на MP3, видеоформаты блокируются.
- **Очередь** — несколько ссылок, последовательная обработка, прогресс, скорость, ETA, отмена.
- **Конвертер** — видео в MP4, MKV, WEBM, MOV; аудио в MP3 320, M4A 256, WAV. H.264 идёт через NVIDIA NVENC с откатом на CPU, WEBM — VP9/Opus.
- **Перетаскивание** — видео или аудио можно бросить прямо в область конвертера.
- **Темы** — по умолчанию как в Windows; можно зафиксировать светлую или тёмную.
- **Языки** — русский и английский.
- **Обложки и теги** — MP3 с обложкой и метаданными ID3.
- **Автообновления** — yt-dlp обновляется сам, AVDock — через GitHub Releases.

## Установка

В каждом релизе два варианта:

1. **Установочная версия** — `AVDock_x.y.z_x64-setup.exe`.
   Ставится без прав администратора в `%LOCALAPPDATA%\Programs\AVDock`, создаёт ярлык в меню Пуск. ffmpeg уже внутри.

2. **Портативная версия** — `AVDock_x.y.z_portable.zip`.
   Распаковать в любую папку и запустить `AVDock.exe`. Рядом должны лежать `resources\ffmpeg.exe` и появится папка `data`.

Инсталлятор не подписан сертификатом, поэтому SmartScreen может показать предупреждение: «Подробнее» → «Выполнить всё равно».

## Где хранятся данные

Приложение старается держать всё рядом с собой:

- портативная версия — в папке `data` рядом с `AVDock.exe`;
- установленная — в `%LOCALAPPDATA%\Programs\AVDock\data`;
- если папка программы недоступна для записи (например, `Program Files`), используется `%APPDATA%\com.exslayder.avdock`.

Внутри:

```
data\settings.json      настройки
data\bin\yt-dlp.exe     yt-dlp
data\tmp\               временные файлы загрузки
```

## Обновления

- **yt-dlp** проверяется при запуске и обновляется автоматически в `data\bin`. Это отдельный движок, он обновляется сам, без переустановки приложения.
- **AVDock** раз в запуск проверяет GitHub Releases (если включено автообновление) и ставит новую версию через инсталлятор. Папка `data` при обновлении сохраняется.
- Отключить автообновление можно в настройках; там же есть ручная проверка yt-dlp и AVDock.

## Стек

| Слой | Технология |
|---|---|
| UI | React 19 + TypeScript + Vite 8 |
| Иконки | Lucide |
| Анимации | Motion + anime.js |
| График скорости | Recharts |
| Шрифт | системный: Segoe UI на Windows, SF Pro на macOS |
| Оболочка | Rust + Tauri 2 |
| Скачивание | `yt-dlp.exe` как отдельный процесс |
| Медиа | `ffmpeg.exe` в ресурсах сборки |

## Разработка

Требуется Rust (MSVC), Node.js 20+ и ffmpeg в `src-tauri/resources/ffmpeg.exe`.

```powershell
cd F:\Python\tools\ai\opencode\avdock
npm install

# если ffmpeg ещё нет — скачать готовую сборку
powershell -ExecutionPolicy Bypass -File scripts/get-ffmpeg.ps1

npm run tauri dev
```

Проверки:

```powershell
npm run build                # tsc + vite
cd src-tauri; cargo check    # Rust
```

## Сборка

```powershell
npm run tauri build
```

Без ключа обновлений сборка создаст только инсталлятор. Для полного набора с файлами обновления:

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content .tauri-keys/avdock.key -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
npm run tauri build
```

Результат:

- `src-tauri\target\release\bundle\nsis\AVDock_x.y.z_x64-setup.exe`
- `src-tauri\target\release\bundle\nsis\AVDock_x.y.z_x64-setup.exe.sig` — подпись для автообновления
- `src-tauri\target\release\avdock.exe` (для портативной сборки)

Готовые файлы также копируются в `build\` (папка в `.gitignore`).

## Релиз

1. Поднять версию в `package.json`, `src-tauri/Cargo.toml` и `src-tauri/tauri.conf.json`.
2. Закоммитить и поставить тег:

   ```powershell
   git tag v0.1.0
   git push origin main --tags
   ```

3. GitHub Actions `Release` соберёт подписанный инсталлятор, обновление и портативный архив, затем создаст релиз с файлами:
   - `AVDock_x.y.z_x64-setup.exe`
   - `AVDock_x.y.z_x64-setup.exe.sig` — подпись обновления
   - `AVDock_x.y.z_portable.zip`
   - `latest.json` для автообновления

Workflow запускается по тегу `v*` или вручную через `workflow_dispatch`. Еженедельной сборки больше нет: приложение не нужно пересобирать по расписанию, а yt-dlp обновляется на стороне пользователя сам.

Для CI в настройках репозитория должны быть секреты:

- `TAURI_SIGNING_PRIVATE_KEY` — содержимое `.tauri-keys/avdock.key`;
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — пароль ключа (пустой, если ключ без пароля).

Приватный ключ в репозиторий не попадает: папка `.tauri-keys` в `.gitignore`. Если ключ потерять, автообновления перестанут проверяться подписью.

Endpoint обновлений: `https://github.com/Exslayder/A_V_Downloader/releases/latest/download/latest.json`.

## Структура

```
avdock/
├─ src/                      React UI
│  ├─ components/            Sidebar, Downloads, Converter, SettingsPage, SpeedChart
│  ├─ lib/                   api.ts, i18n.tsx, format.ts, types.ts
│  ├─ App.tsx
│  └─ styles.css
├─ src-tauri/
│  ├─ src/
│  │  ├─ lib.rs              команды Tauri и запуск
│  │  ├─ state.rs            состояние, очередь, события
│  │  ├─ queue.rs            последовательный воркер загрузок
│  │  ├─ download.rs         запуск yt-dlp, разбор прогресса
│  │  ├─ convert.rs          ffmpeg: форматы, NVENC/CPU, прогресс
│  │  ├─ bins.rs             поиск и обновление yt-dlp, ffmpeg
│  │  ├─ updates.rs          обновления yt-dlp и приложения
│  │  ├─ paths.rs            data рядом с exe, откат в профиль
│  │  └─ settings.rs         settings.json
│  ├─ resources/             ffmpeg.exe (не хранится в git)
│  ├─ capabilities/
│  └─ tauri.conf.json
├─ scripts/get-ffmpeg.ps1
├─ public/theme-init.js
├─ package.json
└─ .github/workflows/release.yml
```

## Оформление

Дизайн собран по методу [unslop](https://github.com/heydamir/unslop) Дамира Соколовского (CC BY 4.0): аудит интерфейса, направление от продукта и девять проходов — отступы, цвет, иконки, текст, типографика, сетка, детали, визуалы, анимация.

- Палитра: нейтральный графит и фиолетовый акцент.
- Типографика: системный шрифт, веса 400–600.
- Иконки: Lucide.
- Анимации: Motion и anime.js, с учётом `prefers-reduced-motion`.

## Отличия от старой версии

- Настройки не переносятся: новый формат и папки по умолчанию (Музыка / Видео).
- Чистятся только обложки, относящиеся к скачанным MP3, а не все картинки в папке.
- Конвертер не перезаписывает исходник, если выбран тот же формат.
- Обновление приложения — инсталлятор Tauri, а не самописный `.bat`.
- Консольное окно не появляется.
- Повторный запуск не создаёт второе окно: приложение фокусирует уже открытое.
