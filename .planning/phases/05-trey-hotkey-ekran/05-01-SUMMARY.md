---
phase: 05-trey-hotkey-ekran
plan: 01
subsystem: desktop-integration
tags: [tray, sni, tray_manager, window_manager, hotkey, config-toml, flutter]

requires:
  - phase: 04-nastroyki-brauzer-modeley
    provides: settings form, HF browser, download manager, 04-REVIEW-FIX UI baseline
provides:
  - SNI tray with 6-item menu, close-to-tray and honest exit
  - state-driven tray icon on the existing status poll
  - read-only hotkey card in the Settings tab
  - phase-4 UI legacy closure
affects: [desktop integration, verification phase 05]

tech-stack:
  added: [tray_manager 0.9.0]
  patterns: [plugin seam with interface+fake, composition-root bootstrap outside core/]

key-files:
  created:
    - app/lib/core/tray.dart
    - app/lib/core/window_shell.dart
    - app/lib/tray_bootstrap.dart
    - app/lib/core/config_path.dart
    - app/lib/features/settings/hotkey_card.dart
    - tools/gen_tray_icons.dart
    - app/assets/tray/idle.png
    - app/assets/tray/recording.png
  modified:
    - app/lib/main.dart
    - app/lib/core/providers.dart
    - app/lib/core/daemon_engine.dart
    - app/lib/l10n/app_ru.arb

key-decisions:
  - "SNI click deviation accepted 2026-11-22: Linux panels keep icon clicks; show/hide is a menu item"
  - "recording and transcribing share the red icon; idle/disconnected is blue"
  - "destroy() only in the Exit menu item; close button is vetoed and hides"

requirements-completed: [TRAY-01, TRAY-02, HK-01, HK-02]
status: complete
---

# Phase 05 Plan 01: Трей + хоткей-экран Summary

**SNI-трей с 6-пунктовым меню и close-to-tray поверх замороженного демона, иконка на существующих 100 мс тиках, read-only HK-карточка в «Настройках»**

## Performance

- **Duration:** (filled at final gate)
- **Started:** 2026-11-22T15:37:06Z
- **Completed:** (filled at final gate)
- **Tasks:** 6
- **Files modified:** (filled at final gate)

## Live tracer smoke (Task 2, 2026-11-22, Plasma 6 Wayland)

Executed against the live daemon (`/run/user/1000/dicta.sock`, status smoke exit 0,
state idle). App launched with `flutter run -d linux`, PID 1178010, then exited
through the menu; no process left behind.

- **Tray icon:** SNI name `org.kde.StatusNotifierItem-1178010-1` registered;
  `IconPixmap` 22×22 with `#35689F` blue pixels; `ToolTip` = "Dicta" (l10n appTitle).
- **Menu:** DBusMenu at `/StatusNotifierItem/Menu` exports all 6 labels —
  «начать запись», «остановить», «отмена», «показать/скрыть окно», «настройки»,
  «выход» (two separators).
- **Commands:** menu Start → daemon `state=recording`; menu Stop → `state=idle`.
  No recording left hanging; daemon untouched otherwise.
- **Show/hide:** menu item toggled window visibility live VISIBLE → HIDDEN → VISIBLE;
  app process and SNI item stayed alive in both hidden phases.
- **Exit:** menu «выход» → app process gone, SNI name unregistered, daemon still
  idle. `Lost connection to device.` in the flutter log, no leftover window.
- **Panel click behaviour:** agent-side physical input is unavailable in this
  Wayland session (no Mutter RemoteDesktop, no per-window screenshot, no AT-SPI
  close button), so the icon-click gesture itself could not be injected. Per the
  accepted SNI deviation the panel owns the click and opens the exported menu;
  the menu path was exercised over DBusMenu (the same path the panel uses).
- **Icon colour on recording:** not switched in this tracer slice — state-to-icon
  wiring is Task 3 (planned expansion). Recorded as expected, not a defect.

## Deviations from Plan

None so far.

## UAT list (for /gsd-verify-work 05, filled at final gate)

1. Tray icon visible on KDE, panel click opens the menu; start/stop live.
2. Window close hides to tray; Exit kills the app.
3. Hotkey card on the live config shows «не задан (off)».
4. Human sign-off: 8 must-NOT + accepted SNI-click deviation (2026-11-22).
5. Hotkey card read-only (config.toml never written).
6. DE table text is reference only, no commands executed.
7. Restart warning shown.
8. Daemon hotkey backstop (HK-02): one trigger, no double stop.
