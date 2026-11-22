---
phase: 05-trey-hotkey-ekran
plan: 01
subsystem: desktop-integration
tags: [tray, sni, tray_manager, window_manager, hotkey, config-toml, flutter]

requires:
  - phase: 04-nastroyki-brauzer-modeley
    provides: settings form, HF browser, download manager, 04-REVIEW-FIX UI baseline
provides:
  - SNI tray with a 6-item menu, close-to-tray and honest exit
  - state-driven tray icon on the existing status poll
  - read-only hotkey card in the Settings tab
  - phase-4 UI legacy closure
affects: [desktop integration, verify-work 05]

tech-stack:
  added: [tray_manager 0.9.0]
  patterns: [plugin seam interface+fake, composition-root bootstrap outside core/, tiny tab notifier]

key-files:
  created:
    - app/lib/core/tray.dart
    - app/lib/core/window_shell.dart
    - app/lib/core/config_path.dart
    - app/lib/tray_bootstrap.dart
    - app/lib/features/settings/hotkey_card.dart
    - tools/gen_tray_icons.dart
    - app/assets/tray/idle.png
    - app/assets/tray/recording.png
    - app/test/fake_tray_service.dart
    - app/test/fake_window_shell.dart
    - app/test/tray_service_test.dart
    - app/test/window_shell_test.dart
    - app/test/config_path_test.dart
    - app/test/hotkey_card_test.dart
  modified:
    - app/lib/main.dart
    - app/lib/core/providers.dart
    - app/lib/core/daemon_engine.dart
    - app/lib/l10n/app_ru.arb
    - app/lib/l10n/generated/app_localizations.dart
    - app/lib/l10n/generated/app_localizations_ru.dart
    - app/lib/features/shell/shell.dart
    - app/lib/features/settings/settings_screen.dart
    - app/lib/features/models/hf_browser.dart
    - app/lib/features/models/model_store.dart
    - app/lib/features/models/models_screen.dart
    - app/lib/features/history/history_screen.dart
    - app/pubspec.yaml
    - README.md
    - README.ru.md

key-decisions:
  - "Accepted SNI deviation (2026-11-22): Linux panels keep icon clicks; show/hide is a menu item — UAT sign-off required"
  - "recording + transcribing share the red icon; idle/disconnected use the blue icon"
  - "destroy() only in the Exit menu item; the X is vetoed and hides"
  - "Hotkey card reads config.toml only; hotkey is never sent through set commands"
  - "Tray rides on the existing 100 ms dictaStatusProvider; no second poll loop"

requirements-completed: [TRAY-01, TRAY-02, HK-01, HK-02]

actuals:
  tokens: 15500
  tasks: 6
  commits: 7

coverage:
  - id: D1
    description: "SNI tray icon + 6-item menu wired to the live daemon"
    requirement: TRAY-01
    verification:
      - kind: unit
        ref: "app/test/tray_service_test.dart#TRAY-02: the X hides the window and never destroys it"
        status: pass
      - kind: other
        ref: "live KDE smoke: DBusMenu exported 6 labels; Start->recording, Stop->idle"
        status: pass
    human_judgment: true
    rationale: "Panel click gesture itself is not injectable in this Wayland session; user signs SNI deviation in UAT"
  - id: D2
    description: "Close-to-tray and honest exit (destroy only from Exit)"
    requirement: TRAY-02
    verification:
      - kind: unit
        ref: "app/test/window_shell_test.dart#close request hides without destroying"
        status: pass
      - kind: other
        ref: "live smoke: Exit unregistered SNI and ended the process"
        status: pass
    human_judgment: false
  - id: D3
    description: "State-driven icon on existing ticks, red on recording/transcribing"
    requirement: TRAY-01
    verification:
      - kind: unit
        ref: "app/test/tray_service_test.dart#icon follows status transitions only"
        status: pass
    human_judgment: false
  - id: D4
    description: "Read-only hotkey card with DE table and restart warning"
    requirement: HK-01
    verification:
      - kind: unit
        ref: "app/test/hotkey_card_test.dart#live-like config shows off, restart warning and DE table"
        status: pass
    human_judgment: false
  - id: D5
    description: "No app-side global hotkey registration"
    requirement: HK-02
    verification:
      - kind: other
        ref: "grep hotkey_manager over pubspec/lock/lib = 0"
        status: pass
    human_judgment: false
  - id: D6
    description: "Phase-4 UI legacy residual closed"
    verification:
      - kind: unit
        ref: "app/test/history_screen_test.dart#long entries collapse to 3 lines and expand on tap"
        status: pass
    human_judgment: false

duration: 36min
completed: 2026-11-22
status: complete
---

# Phase 05 Plan 01: Трей + хоткей-экран Summary

**SNI-трей с 6-пунктовым меню и close-to-tray поверх замороженного демона; иконка на существующих 100 мс тиках; read-only HK-карточка в «Настройках»; закрыто UI-наследие фазы 4**

## Performance

- **Duration:** 36 min
- **Started:** 2026-11-22T15:37:06Z
- **Completed:** 2026-11-22T16:13:29Z
- **Tasks:** 6/6
- **Files modified:** 37
- **Test suite:** 170 tests, all passing (24 inherited files + 6 new suites)

## Coverage of requirements

| Requirement | Tasks | Evidence |
|---|---|---|
| TRAY-01 (tray + icon) | Task 1, 2, 3 | FakeTrayService icon/menu routing; live KDE SNI icon + DBusMenu; `iconFor` transitions |
| TRAY-02 (close-to-tray) | Task 1, 2 | FakeWindowShell: close → hide, destroy only from Exit; live Exit ends process |
| HK-01 (hotkey card) | Task 1, 4 | config_path parity tests + hotkey_card widget fixtures; read-only, DE table, restart warning |
| HK-02 (no global hotkey in app) | Task 1-6 gates | `hotkey_manager` grep = 0 in pubspec/lock/lib |

## Accepted deviation — SNI click (UAT sign-off required)

The literal TRAY-01 wording «left click = show/hide window» is not implementable
with `tray_manager` 0.9.x on Linux: no tray-icon click events are reported; the
panel (KDE Plasma 6) keeps the click and opens the exported menu itself. The
accepted implementation is the **«показать/скрыть окно»** menu item. Accepted by
the orchestrator on 2026-11-22; the user must sign this off in UAT item 2 below.

## Live KDE tracer smoke (Task 2, 2026-11-22)

Executed with the live daemon (`/run/user/1000/dicta.sock`, status smoke exit 0,
state idle) via `flutter run -d linux`, PID 1178010. Process exited through the
menu; no app or recording was left behind.

- Tray SNI `org.kde.StatusNotifierItem-1178010-1` registered immediately;
  `IconPixmap` 22×22 with `#35689F` blue pixels; `ToolTip` = «Dicta» (l10n appTitle).
- DBusMenu at `/StatusNotifierItem/Menu` exported all 6 labels: «начать запись»,
  «остановить», «отмена», «показать/скрыть окно», «настройки», «выход» (two separators).
- Menu Start → daemon `state=recording`; menu Stop → `state=idle`; no hanging recording.
- Show/Hide menu item toggled window visibility live VISIBLE→HIDDEN→VISIBLE; app
  process and SNI item stayed alive while hidden (close-to-tray end state).
- Exit menu item → process gone, SNI unregistered, daemon still idle.
- Icon colour on recording was NOT switched in the tracer slice — state→icon
  wiring arrived in Task 3 (unit-covered); recorded as expected, not a defect.
- **Panel click gesture:** physical input is unavailable in this Wayland session
  (no Mutter RemoteDesktop, no per-window screenshot, no AT-SPI close button),
  so the literal icon click could not be injected. The DBusMenu path the panel
  uses was exercised end-to-end instead. The X-button behaviour is proven by
  `window_shell_test.dart` + the observed hidden-window/alive-process state.

## UI-наследие фазы 4 (Task 5)

Сверено с живым кодом до правки. Уже погашено 04-REVIEW-FIX и НЕ дублировано:
MergeSemantics на слайдерах/сегментах/VRAM-строке, catch-all не-HTTP-исключений,
in-flight disabled+спиннер на сетевых кнопках, отказ ENGINE при выборе модели.

Реализован остаток: `errHttp`/`errDeleteFailed` вместо литералов; заголовок поиска
`modelSearchTitle`, активная модель titleMedium+primary; fetch-ошибка в liveRegion,
МБ-счётчик вне liveRegion; свёртка записей >160 символов с сохранённым
`SelectableText` (HIST-03); ГБ/МБ через .arb-плейсхолдеры; живой VRAM-readout при
drag; межгрупповой ритм 20.

## Task Commits

1. **Task 1 — Wave 0 seams** — `74dcce2` (feat)
2. **Task 2 — live tray tracer** — `661fc0f` (feat)
3. **Task 3 — state icon + settings tab** — `2ff1df7` (feat)
4. **Task 4 — hotkey card** — `593bd79` (feat)
5. **Task 5 — UI legacy closure** — `1d20614` (fix)
6. **Task 6 — README ×2 + final gate** — `docs(05-01)` metadata commit

## Final gate (Task 6, all measured live)

| Gate | Result |
|---|---|
| `flutter test` | 170/170 passed |
| `flutter analyze` | No issues found! |
| `bash tool/cyrillic_gate.sh` | 0 |
| `dart run tool/status_smoke.dart` | SMOKE OK, state=idle |
| `git diff --quiet b81178b HEAD -- crates/ adapters/ systemd/` | RUST-FROZEN |
| implements DictaEngine / TrayService / WindowShell | 1 / 1 / 1 |
| `hotkey_manager` in pubspec+lock+lib | 0 (HK-02) |
| `tray_manager: 0.9.0` pin | 1 (only new dependency; lock diff = tray_manager chain only) |
| Icon regeneration | ICONS-DETERMINISTIC (byte-for-byte) |
| `tray_manager` in lib/ / test/ | 1 / 0 |
| `$error` in daemon_engine.dart | 0 |
| README anchors EN / RU | 1 / 1 |
| 24 .arb anchor keys | 24 |
| Timer/Stopwatch/Socket in tray code | 0 |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] HK card stayed blank on an unreadable config**
- **Found during:** Task 4
- **Issue:** Riverpod 3 keeps a previous error attached to `AsyncLoading`, so
  `when(error:)` never fired; a nested FutureProvider also hit a disposal race.
- **Fix:** render the error card on `config.hasError`; read the daemon settings
  reply directly from `engineProvider` (same source, same primary config_path
  / fallback semantics). No plan-visible behaviour changed.
- **Files:** `app/lib/features/settings/hotkey_card.dart`
- **Verification:** `hotkey_card_test.dart` unreadable-path case.

**2. [Rule 3 - Blocking] `errHttp` generated with a String placeholder**
- **Found during:** Task 5
- **Issue:** generated `errHttp(String code)` rejected the int status code.
- **Fix:** pass `error.statusCode.toString()` at both call sites.

**3. [Accepted limitation] Literal panel icon click not injected**
- **Found during:** Task 2 (live smoke)
- **Issue:** Wayland session exposes no physical-input backend for the agent.
- **Fix:** exercised the same DBusMenu contract the panel uses; unit tests cover
  close-to-tray. Flagged in UAT item 2 for human sign-off.

**Total deviations:** 3 auto-fixed/accepted (2 code fixes, 1 documented limitation).
**Impact:** no protocol, Rust, or config-write changes; all gates green.

## Issues Encountered

None blocking. No stubs or skipped tests left behind.

## UAT checklist for /gsd-verify-work 05

1. Иконка dicta видна в трее KDE сразу после старта приложения.
2. Клик по иконке открывает меню (зафиксировать фактическое поведение левой/правой кнопки панели) и пользователь подписывает принятое SNI-отклонение (2026-11-22).
3. Меню: «показать/скрыть окно» работает; start/stop/cancel доходят до демона.
4. Иконка краснеет при записи/распознавании и синеет в idle (переходы, без 10 Гц-спама).
5. Крестик окна: окно скрылось, приложение живо в трее; «выход» полностью завершает процесс.
6. HK-карточка: текущее evdev-значение или «не задан (off)» на живом конфиге, DE-таблица, предупреждение о рестарте демона; конфиг только читается.
7. Сверка 8 must-NOT из CONTEXT.md (без глобального хоткея в приложении, без записи config.toml, без правок Rust и т.д.).
8. Демон-хоткей при живом приложении: одно срабатывание, без двойного stop (HK-02 backstop).

## Next Phase Readiness

Phase 05 complete and ready for `/gsd-verify-work 05`. UAT items 2 and 8 need
human sign-off; all automated gates are green.

---
*Phase: 05-trey-hotkey-ekran*
*Completed: 2026-11-22*
