# Phase 5 Validation Strategy: Трей + хоткей-экран

```yaml
phase: 5
phase_name: Трей + хоткей-экран
requirements: [TRAY-01, TRAY-02, HK-01, HK-02]
status: draft
```

## Requirement → Verification Map

| REQ | Вердикт edge-probe | Резолюция | Проверка |
|---|---|---|---|
| TRAY-01 | unclassified → mixed | explicit (шов-тесты) + backstop (живой трей глазами) | explicit: FakeTrayService фиксирует setIcon-вызовы по состояниям (idle/recording/transcribing), menu-callback'и вызывают start/stop/cancel существующего DictaControlNotifier (commandInFlight-сериализация уже жива); backstop: живой трей на KDE — иконка в панели, меню открывается кликом панели (SNI: клики по иконке tray_manager на Linux НЕ сообщает — панель сама открывает меню, docs pub.dev; ПРИНЯТО оркестратором 2026-11-22 как реализация «левый клик = доступ к окну»), пункты меню работают |
| TRAY-02 | unclassified → mixed | explicit (юнит шва) + backstop (живое закрытие) | explicit: FakeWindowShell — onWindowClose-ветка даёт hideCount==1 && destroyCount==0; destroy вызывается ТОЛЬКО из пункта меню «выход»; backstop: живое закрытие окна → приложение живо в трее; выход только меню |
| HK-01 | unclassified → explicit | explicit (fixture-тесты) | config-парсинг: чистая функция построчного regex `^\s*hotkey\s*=\s*"([^"]*)"` на fixture-текстах (строка есть / нет / off / с пробелами); путь конфига: settings.configPath из снимка ДЕМОНА (первичный, settings.dart:110) + resolveConfigPath fallback (DICTA_CONFIG → ~/.config/dicta/config.toml, паритет config.rs:173-178); widget-тест карточки: комбинация или «не задан (off)», DE-таблица (KWin Alt+V / Hyprland / Sway / GNOME — статичная справка), предупреждение «применится после рестарта демона» видимы |
| HK-02 | unclassified → explicit | explicit (гейт отсутствия) | `grep -rn "hotkey_manager" app/pubspec.yaml app/pubspec.lock app/lib/ | wc -l` == 0; в pubspec НЕ появляется ни одного хоткей-пакета; приложение не регистрирует хоткеи (grep HotKey/GrabKeyboard по lib/ пуст вне комментариев) |

## Edge-probe RAW output (для аудита)

coverage: applicable 4, unclassified 4 → все резолвлены оркестратором в таблице выше (explicit/backstop по канону фаз 2-4).

## Validation Architecture (из 05-RESEARCH.md, HIGH confidence)

- **TRAY-01**: unit-тесты через FakeTrayService (канон fake_download_engine.dart:11-83: сценарий+гейты+счётчики вызовов): (а) смена состояния поллера → setIcon с соответствующим asset-именем; (б) пункты меню start/stop/cancel вызывают существующие методы контроля; (в) пункт «настройки» открывает окно и переключает вкладку (shell-Notifier). Живой SNI — UAT.
- **TRAY-02**: FakeWindowShell с hideCount/destroyCount; тест закрытия: preventClose-ветка → hide, не destroy; destroy только из меню-колбека. Живое закрытие — UAT.
- **HK-01**: config_path_test.dart — парсинг fixture-текстов verbatim (есть `hotkey = "ctrl+alt+v"` → извлечено; нет строки → off; `hotkey = "off"` → off; ведущие пробелы); hotkey_card widget-тест с фикстурой.
- **HK-02**: гейт-команда в автоматической батарее плана (HEAD=0 EXPECT==0 на pubspec+lock+lib).
- **Иконки**: tools/gen_tray_icons.dart — детерминированный генератор (ZLibEncoder+CRC32, без системных зависимостей); PNG коммитятся; тест: PNG-заголовок валиден (сигнатура \x89PNG) и размер ≠ 0.
- **Порядок (Waves)**: Wave 0 — швы+фейки+генератор иконок+.arb; Wave 1 TRACER — живой трей с меню на живом демоне (иконка idle); Wave 2 — иконка по состоянию + вкладка из меню + HK-карточка; Wave 3 — UI-наследие фазы 4 (один задачей, канон T8 фазы 4); финальный гейт.
- **Manual UAT (обязательные live-пункты)**: (1) трей виден на KDE, меню открывается кликом панели, start/stop живьём, иконка меняется на записи; (2) закрытие окна → жив в трее, «выход» убивает; (3) HK-карточка на живом конфиге (hotkey отсутствует → «не задан»); (4) подпись must-NOT (8 запретов CONTEXT) + принятие отклонения SNI-клика (левый клик = меню панели, пункт «показать/скрыть»).

## Per-Task Verification Map

Привязка к задачам 05-01-PLAN.md (канон фазы 2: binding в ревизии чекера, ревизия 1 от 2026-11-22).

| Task | Requirement / обязательство | Проверка (05-01-PLAN.md) |
|---|---|---|
| Task 1 (Wave 0) | швы TrayService/WindowShell + фейки, tray_manager 0.9.0 пин, PNG-генератор + 2 ассета, 24 .arb-ключа, residual runtimeType (daemon_engine.dart:61) | Task 1 <automated>: пин [HEAD=0 EXPECT==1], assets [HEAD=0 EXPECT==1], PNG-сигнатура 89504e470d0a1a0a, .arb-счётчик [HEAD=0 EXPECT==24], $error [HEAD=1 EXPECT==0], implements TrayService/WindowShell == 1, батарея/analyze/cyrillic |
| Task 2 (Wave 1, TRACER) | TRAY-01/TRAY-02 explicit (unit через фейки) + backstop (живой трей на живом демоне) | Task 2 <automated>: tray_service_test/window_shell_test + батарея; plugin-изоляция (tray_manager lib/ == 1, test/ == 0); живой SMOKE на KDE с гашением (unit+backstop, результаты в SUMMARY) |
| Task 3 (Wave 2) | TRAY-01 иконка по состоянию (гард переходов, transcribing→recording) + вкладка из меню (selectedTabNotifier) | Task 3 <automated>: iconCalls-сценарии, _selected [HEAD=4 EXPECT==0], rail == 4 пункта, свой полл [HEAD=0 EXPECT==0], батарея/analyze/cyrillic |
| Task 4 (Wave 2) | HK-01 (config.toml read-only, DE-таблица, предупреждение о рестарте) | Task 4 <automated>: config_path_test (паритет config.rs:173-178 verbatim) + hotkey_card_test (фикстуры), writeAs/openWrite [HEAD=0 EXPECT==0], set('hotkey') [HEAD=0 EXPECT==0], батарея/analyze/cyrillic |
| Task 5 (Wave 3) | UI-наследие фазы 4 одним задачей (канон T8; сверка живого кода + допил остатка) | Task 5 <automated>: 'HTTP ${error.statusCode}' [HEAD=2 EXPECT==0], 'delete failed' [HEAD=1 EXPECT==0], '} GB'/'} MB' [HEAD=2 EXPECT==0], modelSearchTitle/histExpand [HEAD=0 EXPECT>=1], батарея/analyze/cyrillic |
| Task 6 (Wave 4) | финальные гейты + HK-02 + README ×2 + SUMMARY с UAT-листом | Task 6 <automated>: полная батарея + analyze + cyrillic + status_smoke; контракты git diff --quiet b81178b HEAD -- crates/ adapters/ systemd/ пусты; hotkey_manager [HEAD=0 EXPECT==0] (HK-02); implements DictaEngine/TrayService/WindowShell == 1 каждый; детерминизм иконок (перегенерация байт-в-байт); README-якоря [HEAD=0 EXPECT==1] каждый |

## Sampling

- Поллер статуса: уже 100 мс (фаза 1) — трей-иконка едет на тех же тиках, НОВОГО полл-цикла не заводить (must-NOT 7 по аналогии PATTERNS).
- Иконки: смена только на переходах состояния (setIcon идемпотентен по имени — гард от 10 Гц-спама).
