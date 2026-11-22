# Stack Research

**Domain:** Flutter desktop app (Material You) поверх существующего Rust-демона диктовки + Windows-каркас
**Веха:** v1.1 «Dicta App» (dicta-app + Windows skeleton branch)
**Researched:** 2026-11-22
**Confidence:** HIGH по версиям (все сверены с pub.dev / crates.io / docs.flutter.dev 2026-11-22); дайджесты закэшированы в research-store с тиром MEDIUM (websearch + cross-check по официальным источникам)

---

## Центральное решение: мост Flutter ↔ Rust-ядро

Вопрос сформулирован как «flutter_rust_bridge vs сокет vs named pipes». Ответ по каждому пункту, с учётом ограничений вехи (протокол сокета и демон на Linux не трогаем):

| Подход | Вердикт | Почему |
|--------|---------|--------|
| **Unix-сокет демона, чистый `dart:io`** | ✅ **Linux-путь (основной)** | `dart:io` умеет AF_UNIX нативно: `Socket.connect(InternetAddress('/run/user/1000/dicta.sock', type: InternetAddressType.Unix), 0)`. Ноль новых зависимостей. Протокол JSON-строка-запрос/JSON-строка-ответ уже существует, демон держит нескольких клиентов одновременно (dictactl + dicta-ui + dicta-settings). Живой уровень сигнала и состояние уже приходят в `{"cmd":"status"}`-ответе (`level: 0..1`, `state`, `seconds`, `text`) — дашборд и level-meter в приложении это просто поллинг status, ровно как dicta-ui делает каждые 100 мс. |
| **Named pipe (Windows) + `dart_ipc`** | ✅ **Windows-путь (каркас)** | FlClash-доказанная схема: на десктопе ядро — отдельный процесс, транспорт — UDS на Linux/macOS, named pipe `\\.\pipe\...` на Windows. Rust-сторона: `interprocess` 2.4.8 (local sockets = named pipes на Windows / UDS на Unix, sync + tokio). Dart-сторона: `dart_ipc` 2.0.2 — чистый Dart + win32 FFI, socket-совместимый API, один и тот же JSON-line протокол dictad переносится на Windows без изменений схемы. |
| **flutter_rust_bridge 2.13.0** | ⛔ **НЕ в этой вехе** (отложить) | FRB v2 зрелый (стабилен 2.13.0, активные beta 2.14, Flutter Favorite, FlClash использует его для тонкого Rust-хелпера) — но для dicta он решает не ту задачу. FRB вшивает Rust в процесс приложения как DLL (Cargokit/native assets, кодоген): (а) дублирует уже существующий IPC-путь; (б) ломает совместимость вехи — приложение обязано работать с независимо живущим systemd-демоном, оверлеем и dictactl; (в) добавляет кодоген и сборочную сложность ради ~десятка команд, которые уже ходят по сокету. FRB имеет смысл в будущей вехе (мобильная версия, прямой стриминг аудио в UI без демона). |

**Итоговая стратегия — «IPC везде, платформа меняет только транспорт»**: один `DictaClient` в Dart с двумя реализациями транспорта (UDS через `dart:io` на Linux, named pipe через `dart_ipc` на Windows), поверх существующего JSON-line протокола без его изменения. Это ровно архитектура FlClash и ровно та, что у dicta уже есть.

---

## Recommended Stack

### Core Technologies (Dart/Flutter, `dicta-app/`)

| Technology | Version | Purpose | Why |
|-----------|---------|---------|-----|
| Flutter (stable) | **3.50.0** (Dart 3.14.0, релиз 2026-11-11) | Каркас dicta-app, Linux + Windows | Стейбл-канал выходит ежеквартально; Linux (Debian 10–13, Ubuntu 20.04–24.04) и Windows 10/11 — Supported + CI-tested платформы. Ровно на 3.50/Dart 3.14 вышло поколение leanflutter-плагинов (tray_manager 0.9.0, hotkey_manager 0.4.0, window_manager 0.7.0 — все требуют Flutter 3.50 + nativeapi 0.6.0) — весь стек выровнен одной версией. |
| `flutter_riverpod` | **3.5.0** (Dart ≥3.12) | State management | Дизайн-паритет с FlClash (у него Riverpod). Riverpod 3.0 стабилен с 2025-09, линейка 3.4–3.5 активно патчится (2026-07…11). Riverpod 3: `Notifier`/`AsyncNotifier` — идеальны для состояния демона (idle/recording/transcribing), истории диктовок и провайдера соединения. |
| `dynamic_color` | **2.1.0** (material-foundation, 2026-08) | Material You / динамический цвет на десктопе | Единственный официальный пакет динамического цвета: Linux — XDG Desktop Portal Settings (Plasma 6 отдаёт accent color) или GTK `@theme_selected_bg_color`; Windows — системный accent color. Плюс harmonization ColorScheme. Даром: тёмная/светлая тема из `ColorScheme.fromSeed`. |
| `tray_manager` | **0.9.0** (leanflutter, 2026-11) | Трей: StatusNotifierItem (Linux), NotifyIcon (Windows) | Linux-трей = StatusNotifierItem через D-Bus — работает в KDE Plasma 6 из коробки, `libayatana-appindicator` больше не нужен. Windows: .ico/.png, клики, меню. Используется Spotube в продакшне. Свежайшая версия привязана к Flutter 3.50. |
| `hotkey_manager` | **0.4.0** (leanflutter, 2026-11) | Глобальные хоткеи в приложении | Windows — RegisterHotKey, работает надёжно. Linux: системные хоткеи идут через `org.freedesktop.portal.GlobalShortcuts` (Plasma 6 предоставляет) + требуется `.desktop`-файл с app id. Для dicta это *дополнение*, не замена: на Linux запись без окна уже обеспечена демон-хоткеями (evdev / KWin-скрипт) — приложение лишь подписывает те же действия. |
| `window_manager` | **0.7.0** (leanflutter, 2026-11) | «Свернуть в трей», close-to-tray, скрытие окна | Единый leanflutter-стек (nativeapi 0.6.0): hide()/show() окна при закрытии в трей, позиционирование, skipTaskbar. Известная слабость: на Wayland часть операций ограничена протоколом — для нашей сцены (скрыть/показать окно) достаточно. |
| `dart_ipc` | **2.0.2** (2026-09) | IPC-транспорт приложения | Socket-совместимый API (`bind`/`connect`/`listen`): named pipe `\\.\pipe\dicta` на Windows, UDS на Unix. Чистый Dart + win32 FFI — нет C++-плагин-слоя, нет кодогена. 150 pub points. На Linux можно и без него (`dart:io`), но единый API для обеих платформ упрощает `DictaClient`. |
| `dio` | **5.11.1** (2026-09) | Браузер моделей HF в приложении | Перенос `hf.rs`/`catalog.rs` в Dart: поиск по HF API (`filter=automatic-speech-recognition`, сортировки, пагинация) и скачивание ggml с прогрессом — у dio есть `download()` c `onReceiveProgress` и CancelToken. Проще, чем спавнить curl из Flutter. Логику URL/парсинга (`browse_url`, `tree_url`, `resolve_url`, толерантный JSON) портируем как чистые Dart-функции с тестами. |
| `flutter_localizations` + `intl` (SDK) | в составе Flutter 3.50 | l10n, русский-first | Официальный `gen-l10n` (см. ниже), без сторонних пакетов. |

### Rust-добавления (Windows-ветка, `crates/`)

| Crate | Version | Purpose | Why |
|-------|---------|---------|-----|
| `interprocess` | **2.4.8** (2026-11) | Named pipe (`\\.\pipe\dicta`) как транспорт dictad на Windows | Local sockets: named pipes на Windows / UDS на Unix, стабильная мапа «имя → примитив», sync API + tokio-фича. Позволяет вынести транспорт демона в `cfg(windows)`-модуль, не трогая Linux-путь. |
| `cpal` | **0.18.2** (2026-08) | Захват микрофона на Windows (WASAPI) | Стандартная Rust audio-библиотека: на Windows дефолтный бэкенд WASAPI (Win 10+), на Linux теперь нативный PipeWire-бэкенд (feature `pipewire`, выбор PipeWire > PulseAudio > ALSA). Заменяет `pw-record` в Windows-каркасе, отдаёт те же 16 кГц mono s16 в WAV для существующего `dicta-core::stt`. **Аудио остаётся в Rust-ядре** — уровень сигнала течёт в status-ответе по IPC, как на Linux. |
| `enigo` | **0.6.1** (MSRV 1.85) | Вставка текста на Windows (SendInput) | Windows-бэкенд: statically-linked `windows` crate, `SendInput` + `KEYEVENTF_UNICODE` для Unicode-текста, ноль рантайм-зависимостей. Вызывается из Windows-ветки `inject.rs` взамен uinput/wl-clipboard. API «alpha», но для SendInput-вставки зрелый (1.8M скачиваний/квартал). Альтернатива — прямые вызовы через `windows` crate без зависимости (в `enigo` под капотом тот же код). |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `flutter create --platforms=linux,windows dicta-app` | Каркас приложения | Linux-сборке нужны `ninja-build libgtk-3-dev libx11-dev libxi-dev` (GTK-плагины Flutter + leanflutter nativeapi). |
| `flutter gen-l10n` + `l10n.yaml` | Локализация, русский-first | `arb-dir: lib/l10n`, шаблон — `app_ru.arb` (или `template-arb-file: app_ru.arb`), `preferred-supported-locales: [ru]`, `untranslated-messages-file` для контроля непереведённых. Никаких сторонних l10n-пакетов. |
| `subosito/flutter-action@v2` (2.23.0) | Flutter на GitHub Actions | Windows-сборка: `runs-on: windows-latest`, `flutter-version: 3.50.0` (или `flutter-version-file: pubspec.yaml` — тогда в pubspec должно стоять точное `flutter: 3.50.0`). Rust-часть на том же раннере: `dtolnay/rust-toolchain@stable` + `cargo build --release` (кросс-компиляция из Linux тоже возможна, но линковка whisper/Vulkan проще на виндовом раннере). |
| `actions/upload-artifact@v4` | Дистрибутив Windows-каркаса | Zip папки `build/windows/x64/runner/Release` (+ рядом `dictad.exe`). MSIX (`msix` package) — опционально, для каркаса избыточен. |

## Installation

```bash
# Каркас
flutter create --platforms=linux,windows dicta-app
cd dicta-app

# Core
flutter pub add flutter_riverpod dynamic_color tray_manager hotkey_manager \
  window_manager dart_ipc dio
flutter pub add flutter_localizations --sdk=flutter
flutter pub add intl

# l10n
mkdir -p lib/l10n && touch lib/l10n/app_ru.arb   # шаблон — русский

# Rust (Windows-ветка, из корня workspace)
cargo add interprocess --package dictad
cargo add cpal --package dicta-core
cargo add enigo --package dicta-core
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| Unix-сокет через `dart:io` | `native_socket` 0.4.2 / другие UDS-пакеты | Не нужен: `dart:io` покрывает AF_UNIX; пакеты добавляют native-assets/hooks ради того, что уже есть в SDK. `native_socket` ещё и требует experimental flag. |
| `dart_ipc` (named pipe) | AF_UNIX на Windows (Dart 3.11+) | Dart ≥3.11 умеет AF_UNIX на Windows, но он опирается на драйвер afunix (есть не на всех Win10), не поддерживает datagrams/abstract, и файловая семантика отличается (reparse points). Named pipe — FlClash-проверенный путь. |
| `dart_ipc` (named pipe) | `flutter_rust_bridge` для IPC-сервера как у FlClash rust_api | Имеет смысл, только если появится потребность в богатых in-process Rust API (мобильная веха, прямой аудио-стрим в Dart). Сейчас дублирует существующий демон. |
| `tray_manager` 0.9.0 | `system_tray` 2.0.3 | `system_tray` протух (3 года без релизов, требует `libayatana-appindicator3-dev`); `tray_manager` свежий, на едином nativeapi-ядре, SNI без appindicator. |
| Riverpod 3 (без кодогена) | `riverpod_generator` 4.0.9 + `riverpod_annotation` 4.0.8 | Официальная рекомендация самого Riverpod: кодоген — только если проект уже кодогенит. dicta-app не кодогенит (gen-l10n — отдельный инструмент без build_runner). Ручные `Notifier`-провайдеры проще и не тормозят сборку. |
| `dio` 5.11.1 | `http` 1.6.0 (dart.dev) | `http` достаточен для пары JSON-запросов, но у него нет download-прогресса и таймаутов из коробки — а скачивание моделей с прогрессом/отменой в браузере моделей нужно. |
| `enigo` | прямые `SendInput` через `windows` crate | Если хочется нулевых лишних зависимостей — в `enigo` под каповом тот же код; для каркаса enigo быстрее пишет MVP. |
| Поллинг status (100 мс) | Отдельный event-канал / streaming | Протокол менять нельзя (ограничение вехи). dicta-ui уже поллит 100 мс — проверенная нагрузка. Если позже появится «events»-команда — переход будет локальным в `DictaClient`. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `flutter_rust_bridge` в вехе v1.1 | Дублирует IPC, ломает схему «приложение = ещё один клиент демона», добавляет кодоген/Cargokit | UDS/named-pipe клиент поверх существующего протокола |
| `system_tray` | Не обновлялся 3 года, тянет libayatana-appindicator | `tray_manager` 0.9.0 |
| Dart-аудио-пакеты (`record`, `flutter_sound`, `just_audio` и т.п.) | Мобильно-ориентированы, на Windows идут через fmedia/чужие рантаймы; уровень сигнала уже отдаёт демон | Аудио в Rust-ядре: `cpal` 0.18.2 (WASAPI), level в status-ответе |
| `bitsdojo_window` | Заброшен, Wayland-проблемы | `window_manager` 0.7.0 |
| Редактирование протокола сокета / демона ради приложения | Жёсткое ограничение вехи: Linux-часть обязана работать поверх существующего dictad без изменений протокола | Приложение — равноправный клиент (как dictactl/dicta-ui) |
| Provider/Bloc/GetIt | Лишние споры; выбран дизайн-паритет с FlClash | Riverpod 3.5 |
| `intl`-переводы «руками» / easy_localization | easy_localization тянет свой рантайм и обходит gen-l10n | Официальный `flutter gen-l10n` + ARB |

## Stack Patterns by Variant

**Linux (основная веха):**
- `DictaClient` → `Socket.connect(InternetAddress(cfgSocketPath, type: InternetAddressType.Unix), 0)` (или `dart_ipc.connect`)
- Поллинг `{"cmd":"status"}` 10 Гц → Riverpod-провайдер `DictaState` (state/level/seconds/text) → дашборд/level-meter
- `{"cmd":"start|stop|cancel"}` с кнопок UI; `{"cmd":"set"}` из настроек; `{"cmd":"inject"}` из истории
- Трей `tray_manager` (SNI, Plasma); окно `window_manager` (close→hide)
- Хоткеи: НЕ дублировать демон-хоткеи; hotkey_manager в Linux — опционально через GlobalShortcuts-portal

**Windows-каркас (отдельная ветка):**
- Тот же JSON-line протокол; транспорт — named pipe `\\.\pipe\dicta` (`interprocess` в dictad, `dart_ipc` в приложении)
- Захват: `cpal` (WASAPI) → существующий stt-путь; вставка: `enigo` (SendInput + Unicode)
- Хоткеи: `hotkey_manager` (RegisterHotKey) — здесь это первичный путь записи без окна
- Сборка: только CI (`subosito/flutter-action@v2` на `windows-latest`); локально — Windows-ПК пользователя

**Если позже понадобится мобильная версия или прямой аудио-стрим в UI:**
- Вот тогда `flutter_rust_bridge` 2.13+ — зрелый, FlClash-проверенный мост для тонкого Rust-слоя

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| tray_manager 0.9.0 / hotkey_manager 0.4.0 / window_manager 0.7.0 | Flutter 3.50 / Dart 3.14 / nativeapi 0.6.0 | Все трое вышли в одном поколении (ноябрь 2026) — ставить вместе, не смешивать с 0.6.x/0.3.x версиями (те требуют Flutter 3.47) |
| flutter_riverpod 3.5.0 | Dart ≥ 3.12 | Ок на Dart 3.14; если трогать Riverpod 2.x-примеры из старых туториалов — API заметно отличается |
| cpal 0.18.x | Rust ≥ 1.85; WASAPI — Windows 10 (README crates.io) / Windows 8 (MSRV-таблица) | MSRV 1.85 выше, чем у остального workspace — проверить тулчейн CI; ALSA всё равно нужна на Linux-сборке cpal (пакет libasound2-dev), даже если используется PipeWire-фича |
| interprocess 2.4.8 | Rust ≥ 1.75 | Sync-модель вписывается в thread-архитектуру dictad (tokio не обязателен) |
| flutter-action@v2 `flutter-version-file` | pubspec с точным `flutter: 3.50.0` | Диапазоны (`>=`) не резолвятся action'ом; yq на windows-раннере action ставит сам |
| Dart AF_UNIX на Windows | Dart ≥ 3.11 | Не используем (см. Alternatives), но знать: reparse-point семантика, `File.existsSync` врёт для сокетов |
| dynamic_color 2.1.0 | Linux — XDG portal или GTK theme | На KDE Plasma 6 идёт через портал (org.freedesktop.impl.portal.Settings); на голых WM вернёт null → fallback на seed-цвет, `DynamicColorBuilder` это предусматривает |

## Sources

- docs.flutter.dev (SDK archive, supported platforms) — Flutter 3.50.0, Dart 3.14.0, поддержка Linux/Windows desktop — HIGH (первоисточник)
- pub.dev/packages/flutter_riverpod (+versions, changelog) — Riverpod 3.5.0, riverpod_generator 4.0.9 — HIGH (первоисточник)
- pub.dev/packages/dynamic_color (material-foundation) — 2.1.0, Linux XDG portal/GTK, Windows accent — HIGH
- pub.dev/packages/tray_manager, hotkey_manager, window_manager (leanflutter) — 0.9.0/0.4.0/0.7.0, nativeapi 0.6.0, SNI, GlobalShortcuts-portal — HIGH
- pub.dev/packages/dart_ipc + github.com/monkeyWie/dart_ipc — 2.0.2, named pipe via win32 FFI — HIGH
- crates.io/cpal + RustAudio/cpal releases — 0.18.2, WASAPI/PipeWire — HIGH
- crates.io/interprocess + docs.rs — 2.4.8, local sockets — HIGH
- crates.io/enigo + github.com/enigo-rs/enigo — 0.6.1, SendInput/KEYEVENTF_UNICODE — HIGH
- pub.dev/packages/flutter_rust_bridge + github.com/fzyzcjy/flutter_rust_bridge/releases — stable 2.13.0, beta 2.14.0-beta.2 — HIGH
- StackOverflow «Does Dart support Unix Domain Socket on Windows?» + dart-lang/sdk CHANGELOG 3.11.0 — AF_UNIX в dart:io (Linux давно, Windows с 3.11) — MEDIUM/HIGH
- github.com/subosito/flutter-action + marketplace — v2.23.0, windows-latest workflow — HIGH
- FlClash: chen08209/FlClash (.agents/architecture.md, DeepWiki), wyusgw/FlClash docs/architecture.md, opendeep.wiki rust-bridge page — UDS/named pipe desktop IPC, Riverpod, rust_api через FRB 2.13.0 — MEDIUM (wiki/зеркала репозиториев)
- pub.dev/packages/dio + fluttergo.ai сравнение http vs dio — 5.11.1, download progress — MEDIUM/HIGH
- Дайджесты всех 13 вопросов закэшированы в `research-store` (ключи из research-plan, тир MEDIUM)

---
*Stack research for: dicta v1.1 «Dicta App» — Flutter desktop + Windows skeleton*
*Researched: 2026-11-22*
