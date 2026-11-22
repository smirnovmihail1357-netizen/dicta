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

Task IDs: TBD — привязка к задачам 05-01-PLAN.md выполняется после создания плана (канон фазы 2: binding в ревизии чекера).

## Sampling

- Поллер статуса: уже 100 мс (фаза 1) — трей-иконка едет на тех же тиках, НОВОГО полл-цикла не заводить (must-NOT 7 по аналогии PATTERNS).
- Иконки: смена только на переходах состояния (setIcon идемпотентен по имени — гард от 10 Гц-спама).
