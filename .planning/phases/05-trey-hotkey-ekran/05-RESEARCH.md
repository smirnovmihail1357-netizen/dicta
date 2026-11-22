# Phase 5: Трей + хоткей-экран — Research

**Phase:** 5
**Requirements:** TRAY-01, TRAY-02, HK-01, HK-02
**Researched:** 2026-11-22
**Domain:** Flutter desktop tray (SNI/Linux) + close-to-tray + чтение config.toml
**Confidence:** HIGH (все ключевые факты сверены живыми read-only командами этой сессии: pub.dev README/CHANGELOG tray_manager и window_manager fetched 2026-11-22, pub-cache nativeapi прочитан по строкам, живой config.toml и код app/ прочитаны с номерами строк)

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked decisions (verbatim)

- Протокол демона и Rust-часть (crates/, adapters/, systemd/) ЗАМОРОЖЕНЫ: никаких новых команд, правок dictad/DictaEngine-контракта семантически не вводить. Трей использует ТОЛЬКО существующие команды start/stop/cancel/status/set.
- HK-02 = запрет: приложение НЕ регистрирует глобальный хоткей записи на Linux (он уже в демоне; дубль = гонка двойного stop). Это must-NOT с гейтом.
- Кириллица в lib/ вне generated — только через .arb-ключи (гейт cyrillic_gate).
- Форма ответа демона фиксирована: {"base":..., "panel":bool, "settings":...}; SetReply{note,error,settings} из фазы 4 — note IS DATA.
- UI-SPEC не создаётся: ui-plan-gate вернул block:false (решение оркестратора, автономный режим).
- Декомиссия dicta-settings — фаза 8; его код в этой фазе не трогаем.
- Нулевые РАБОЧИЕ зависимости не обязательны для плагин-слоя: tray-функциональность требует Flutter-плагин tray_manager (выбор STACK-исследования: tray_manager 0.9.0, leanflutter nativeapi-ядро). Это СПЛАНИРОВАННАЯ зависимость вехи, не нарушение; версию закрепить точечно в pubspec. Иные новые пакеты — запрещены.

### Must-NOT (гейты, verbatim)

1. НЕ регистрировать глобальный хоткей в приложении (никаких hotkey_manager-вызовов на Linux; пакет НЕ добавлять в pubspec) — HK-02.
2. НЕ менять протокол/демона/Rust-часть (контракт байт-в-байт к phase-base).
3. НЕ писать config.toml (только чтение; редактирование хоткея — вне приложения, инструкция на экране).
4. НЕ трогать dicta-settings crate.
5. Никаких print/логов payload в lib.
6. Кириллица в lib только через .arb.
7. Без новых пакетов, кроме tray_manager (если research не докажет иное — тогда снова к оркестратору).
8. Без новых rail-пунктов без обоснования (4 пункта закреплены фазой 1).

### Входы из аудитов (обязательства фазы)

1. UI-наследие фазы 4 (04-UI-REVIEW.md, одним задачей): MergeSemantics для слайдеров; локализация ошибок + error-snackbar-скелет; in-flight индикаторы + VRAM-readout live; заголовок поиска + вес модели; liveRegion fetch-ошибок + excludeSemantics МБ-счётчика; свёртка длинных записей; ГБ/МБ → arb-hygiene; ритм формы 12/20-24.
2. daemon_engine.dart:61 `$error` → error.runtimeType.
3. Точка входа трей-инициализации должна учитывать тесты: buildFakeApp/buildTestApp не должны дёргать plugin-каналы.

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TRAY-01 | Трей SNI: start/stop/отмена/настройки/выход в меню, левый клик = показать/скрыть окно, иконка отражает состояние записи | Native API tray_manager 0.9.0 (сниппет ниже); меню через Menu.create + setContextMenuTrigger; иконки через ImageAsset.fromAsset; ВАЖНО: на Linux клики по иконке НЕ приходят (см. Pitfall 1) — «левый клик» = панель сама открывает меню, показать/скрыть — пунктом меню |
| TRAY-02 | Закрытие окна сворачивает в трей, а не убивает приложение; честный выход — пунктом меню | window_manager 0.7.0 legacy: setPreventClose(true) + onWindowClose → hide(), выход = destroy() — подтверждено pub.dev README |
| HK-01 | Экран хоткея показывает текущую комбинацию, путь срабатывания (evdev vs DE-адаптер) и предупреждение «применится после рестарта демона» | Живой формат config.toml (плоский, без секций), regex-парсер `hotkey = "..."` без toml-пакета, статичная DE-таблица из README:223-234 + adapters/README.md |
| HK-02 | Приложение НЕ регистрирует глобальный хоткей записи на Linux | Гейт: hotkey_manager отсутствует в pubspec и lib/ (проверено grep этой сессией); зависимость вообще не добавляется |

</phase_requirements>

## Summary

Фаза технически меньше фазы 4 и почти целиком опирается на уже собранную инфраструктуру: window_manager 0.7.0 уже стоит в pubspec (app/pubspec.yaml:35) и уже привёл в pub-cache общий nativeapi 0.6.1 — тот же C++ core, на котором построен tray_manager 0.9.0 (обе версии требуют nativeapi ^0.6.0, оба пакета leanflutter, verified publisher). Linux-сборка трей-плагина требует libgtk-3/libx11/libxi dev-заголовки — все три подтверждены на этой машине. Активная версия пакета на pub.dev сегодня — 0.9.1 (вышла 2026-11-22, bump nativeapi 0.6.2); locked 0.9.0 валиден и совместим с уже разрешённым nativeapi 0.6.1 — рекомендую pin `tray_manager: 0.9.0` в стиле существующего `window_manager: 0.7.0`.

Главное открытие, меняющее букву TRAY-01: **tray_manager на Linux не сообщает клики по иконке вообще** — ни левый, ни правый (документировано в pub.dev README «Linux still reports no tray icon clicks at all: the panel keeps them and opens the menu itself», повторено в changelog 0.7.0: «Tray icon clicks are **not** reported there — the panel keeps them and opens the menu itself»). На KDE Plasma клик по SNI-иконке открывает экспортированное меню силами самой панели. Поэтому «левый клик = показать/скрыть окно» реализуется как пункт меню «Показать/скрыть окно» в меню, которое панель открывает по клику; событие `onTrayIconMouseDown` на Linux недоступно ни в native, ни в legacy API. Это отклонение от формулировки требования — оркестратор должен подтвердить (см. Open Questions RESOLVED #1).

Иконки: rootBundle/temp-файл НЕ нужен — `ImageAsset.fromAsset('assets/tray/idle.png')` сам резолвит asset в FS-путь `<exe>/data/flutter_assets/<name>` и читает файлом (проверено по исходнику nativeapi_flutter-0.6.1/src/widgets/image_asset.dart:28-51, прочитан живьём из pub-cache). PNG-заготовки: микро-Dart-инструмент в tools/ (dart:io ZLibEncoder + CRC32 ~80 строк, без системных зависимостей — precedent tools/fetch-model.sh) + закоммитить результат в app/assets/tray/{idle,recording}.png (22×22 RGBA). Конфиг: живой ~/.config/dicta/config.toml — плоский TOML без секций, 11 строк, строки `hotkey` в нём НЕТ (значит демон работает с default "off"); хоткей-строка достаётся line-based regex, без toml-пакета.

**Primary recommendation:** один шов `TrayService` (interface в core/tray_service.dart, реализация TrayManagerService — единственное место с plugin-вызовами, fake в test/) + `WindowShell`-обёртка для setPreventClose/onWindowClose/destroy; HK-карточка в существующей вкладке «Настройки» (нового rail-пункта не вводить); все русские строки — через .arb; Pin: `tray_manager: 0.9.0`.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| SNI-трей (иконка, меню, tooltip) | core/tray_service.dart (plugin-вызовы ТОЛЬКО здесь) | features/shell — только вызовы шва | Единый канон швов фазы 4: transport в core/, UI в features/ (shell.dart:3, hf_client.dart:2-3) |
| Смена иконки по состоянию | TrayController (слушает dictaStatusProvider) | TrayService.setIcon(state) | Полл 100 мс уже жив (providers.dart:57-94); маппинг state→asset — чистая функция, тестируется |
| Close-to-tray (preventClose/hide/destroy) | core/window_shell.dart (обёртка над window_manager legacy) | TrayService.menuExit → WindowShell.destroy | plugin-каналы в одном файле, юнит-тест через интерфейс |
| Меню трея (лейблы, колбэки) | TrayService.setMenu + DictaControlNotifier | — | команды start/stop/cancel уже сериализованы в фазе 2 (CONTEXT:22) |
| Чтение config.toml (hotkey) | core/config_path.dart — чистые функции resolveConfigPath + parseHotkey | HK-виджет в features/settings — только отображение | чистые функции = fixture-тесты без dart:io в виджет-тестах |
| HK-01 карточка (DE-таблица, предупреждение) | features/settings/settings_screen.dart | — | rail = 4 пункта (shell.dart:57-78), зона HK — карточка/секция в «Настройках» |
| Гейт HK-02 | verification (grep hotkey_manager) | — | must-NOT, не код |
| Иконки-ассеты | app/assets/tray/*.png + tools/gen_tray_icons.dart | pubspec assets-секция | генерация без системных зависимостей, артефакт в git |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|----------|---------|---------|--------------|
| tray_manager (leanflutter) | **0.9.0 (pin точечно)** | SNI-трей: иконка, tooltip, контекстное меню | Locked CONTEXT:14; nativeapi-ядро общее с window_manager 0.7.0 (pub.dev deps: `nativeapi: ^0.6.0` у обоих — window_manager-0.7.0/pubspec.yaml прочитан); Flutter 3.50/Dart 3.14 requirement совпадает с проектом (Flutter 3.50.1 установлен) |
| window_manager (leanflutter) | 0.7.0 (уже в pubspec, НЕ трогать) | close-to-tray: setPreventClose/onWindowClose/hide/destroy | pubspec.yaml:35; main.dart:7,11 уже на legacy.dart |

### Supporting (уже в проекте, не добавлять)

| Library | Version | Purpose | When to Use |
|----------|---------|---------|-------------|
| flutter_riverpod | 3.5.0 | провайдер шва trayServiceProvider + слушание dictaStatusProvider | вся фаза |
| dart:io (stdlib) | — | чтение config.toml + ZLibEncoder в генераторе PNG | config_path.dart, tools/gen_tray_icons.dart |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| tray_manager 0.9.0 native API (`tray_manager.dart`) | `tray_manager/legacy.dart` (0.5.x мост: `trayManager` singleton, `TrayListener.onTrayIconMouseDown`) | legacy проще по форме, но @Deprecated и «will be removed in a later release» (pub.dev README); native API — будущее пакета; шов TrayService скрывает выбор в любом случае. **Рекомендация: native API** |
| tray_manager 0.9.1 (вышла сегодня 2026-11-22) | bump nativeapi→0.6.2 | не нужен: 0.9.0 совместим с разрешённым nativeapi 0.6.1 (уже в pub-cache); locked-решение CONTEXT:14 = 0.9.0 |
| PNG-ассеты + ImageAsset.fromAsset | Image.fromBase64(константа в Dart) | base64 без файлов вообще, но нечитаемо и не редактируемо дизайнером; ассеты — канон |
| ручное рисование иконок в коде | закоммитить готовые PNG + tools-генератор | артефакт воспроизводим, diff-читаем, валидность PNG фиксируется генератором |

**Установка:**
```yaml
# app/pubspec.yaml — единственное изменение зависимостей фазы:
dependencies:
  tray_manager: 0.9.0   # точечный pin, стиль window_manager: 0.7.0 (pubspec.yaml:35)
# и assets-секция (сейчас её нет — pubspec.yaml:65-67 закомментирован):
flutter:
  assets:
    - assets/tray/
```

## Package Legitimacy Audit

> npm-гейт `gsd-tools query package-legitimacy check` неприменим: он опрашивает npm-реестр, где pub.dev-пакетов естественно нет (вернул SLOP «does-not-exist» для всех — ложное срабатывание кросс-экосистемного конфуза, задокументированного в самом протоколе). Легитимность проверена напрямую на pub.dev 2026-11-22.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| tray_manager | pub.dev (verified publisher **leanflutter.dev**) | с 2020 (0.0.1), 0.9.0/0.9.1 актуальны | 287k/wk | github.com/leanflutter/tray_manager | OK | Approved — pin 0.9.0 |
| window_manager | pub.dev (verified publisher leanflutter.dev) | годы | >1M/wk (1.1k likes) | github.com/leanflutter/window_manager | OK | Уже в pubspec, не трогать |
| nativeapi / nativeapi_flutter / cnativeapi | pub.dev | 0.6.x актуальная линия | транзитивные | github.com/libnativeapi/nativeapi | OK | Транзитивные (приходят сами), pub-cache уже содержит 0.6.1 |

**Packages removed due to [SLOP] verdict:** none (npm-SLOP — артефакт не того реестра, см. примечание выше).
**Packages flagged as suspicious [SUS]:** none.

Postinstall-скриптов нет — pub.dev-пакеты, не npm; native-часть собирается через Flutter build hook (`app/.dart_tool/hooks_runner/cnativeapi/` уже живой precedent — собран window_manager).

## Architecture Patterns

### System Architecture Diagram

```
            ┌─────────────────────────── Linux/KDE (SNI) ───────────────────────────┐
            │  Plasma panel (StatusNotifierItem host)                              │
            │    клик по иконке → панель САМА открывает экспортированное меню        │
            │    (click events в приложение НЕ приходят — tray_manager 0.9.x)      │
            └───────▲───────────────────────────────────────────────────────────────┘
                    │ D-Bus SNI (иконка/tooltip/меню)
┌───────────────────┴───────────────────────────────────────────────────────────┐
│ dicta-app (Flutter)                                                           │
│                                                                               │
│  main.dart ──► tray bootstrap (только в prod-main, НЕ в buildFakeApp)          │
│      │            │                                                           │
│      │      core/tray_service.dart  ── ЕДИНСТВЕННОЕ место plugin-вызовов       │
│      │        TrayService (interface) ◄── FakeTrayService (test/)             │
│      │        TrayManagerService → TrayIcon.create() / Menu.create() /         │
│      │                            setContextMenuTrigger / ImageAsset          │
│      │            ▲ listens dictaStatusProvider (100 мс полл)                 │
│      │            │   state idle/recording/transcribing → icon idle/recording │
│      │            │                                                           │
│      │      core/window_shell.dart ── window_manager legacy:                   │
│      │        setPreventClose(true); onWindowClose → hide(); exit → destroy() │
│      │            ▲ menu callbacks (показать/скрыть, выход)                  │
│      │            │                                                           │
│  меню ─► DictaControlNotifier ─► DictaEngine ─► UDS ─► dictad (ЗАМОРОЖЕН)       │
│              start/stop/cancel — существующие команды, protocol untouched     │
│                                                                               │
│  features/settings (вкладка «Настройки», rail-пункт #4)                        │
│      HK-карточка ◄─ core/config_path.dart (чистые функции)                     │
│          resolveConfigPath: DICTA_CONFIG → ~/.config/dicta/config.toml        │
│          parseHotkey: line-based regex `^hotkey = "..."` → "off" default       │
│          + статичная DE-таблица (KWin/Hyprland/Sway/GNOME/evdev)               │
│          + предупреждение «применится после рестарта демона»                   │
└───────────────────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure (только новое)

```
app/
├── assets/tray/           # idle.png, recording.png (22×22 RGBA, в git)
├── lib/core/
│   ├── tray_service.dart   # TrayService interface + TrayIconState enum; БЕЗ plugin-импортов в interface-части
│   ├── tray_manager_service.dart  # реализация: TrayIcon.create/Menu/create — все plugin-вызовы здесь
│   ├── window_shell.dart   # WindowShell interface + WindowManagerShell (setPreventClose/onWindowClose/hide/destroy)
│   └── config_path.dart    # resolveConfigPath + parseHotkey — чистые функции, dart:io только в resolve
├── lib/features/settings/
│   └── hotkey_card.dart    # HK-01 карточка (или секция внутри settings_screen.dart — решение планёра)
└── test/
    ├── fake_tray_service.dart
    ├── tray_service_test.dart      # TRAY-01: маппинг state→icon, menu callbacks
    ├── window_shell_test.dart      # TRAY-02: close→hide, exit→destroy
    ├── config_path_test.dart       # HK-01: fixtures
    └── hotkey_card_test.dart       # HK-01: widget-тест карточки
tools/
└── gen_tray_icons.dart    # микро-PNG-генератор (dart:io, без системных зависимостей)
```

### Pattern 1: TrayService-шов (канон фазы 4: HfClient/DownloadEngine)

**What:** interface в core/, реализация с plugin-вызовами рядом, fake в test/, override через ProviderScope — точная копия hfClientProvider/downloadEngineProvider (providers.dart:21-25).

```dart
// core/tray_service.dart — plugin-каналы НЕ дёргаются из interface
enum TrayIconState { idle, recording }   // transcribing — опционально третьей

abstract interface class TrayService {
  Future<void> start();               // создать TrayIcon, повесить меню, setVisible
  Future<void> setIconState(TrayIconState state);
  void onMenuShowHide(void Function() cb);   // «Показать/скрыть окно»
  void onMenuToggleRecord(void Function() cb); // start/stop по текущему статусу
  void onMenuCancel(void Function() cb);
  void onMenuSettings(void Function() cb);
  void onMenuExit(void Function() cb);
  Future<void> dispose();
}

// core/providers.dart
final trayServiceProvider = Provider<TrayService>((_) => TrayManagerService());
```

**When to use:** любой plugin-пакет в этом проекте (канон APP-03: «UI не знает о транспорте», shell.dart:3).

### Pattern 2: Инициализация трея только в prod-main

**What:** tray bootstrap вызывается из `main()` после `runApp`-setup, а buildFakeApp/test-сборки никогда не конструируют TrayManagerService (в widget-тестах platform-channel нет — CONTEXT:25,26 верифицировано: buildFakeApp сейчас единственный scaffold, fake_dicta_engine.dart:162-204).

```dart
// main.dart (prod-точка входа; тесты main() не запускают)
runApp(const ProviderScope(child: DictaApp()));
// после runApp: bootstrap трея читает статус-провайдер и вешает меню.
// Окно уже инициализировано (main.dart:11 windowManager.ensureInitialized).
```

### Pattern 3: Минимальный рабочий tray_manager 0.9.0 native API (Linux)

**Источник: pub.dev/packages/tray_manager README, fetched 2026-11-22 (VERIFIED):**

```dart
import 'package:tray_manager/tray_manager.dart';

final trayIcon = TrayIcon.create()!;                    // ДЕРЖАТЬ ссылку: GC = иконка исчезает
trayIcon.icon = ImageAsset.fromAsset('assets/tray/idle.png');  // asset-путь, НЕ файл и НЕ rootBundle
trayIcon.setTooltip('dicta');

final menu = Menu.create()!;
final item = MenuItem.createWithLabelAndType('Show Window', MenuItemType.normal)!;
item.addListener((event) {
  if (event is MenuItemClickedEvent) { /* показать/скрыть */ }
});
menu.addItem(item);
menu.addSeparator();
menu.addItem(MenuItem.createWithLabelAndType('Exit App', MenuItemType.normal)!);
// ... остальные пункты: start/stop, cancel, settings

trayIcon.setContextMenu(menu);
// ОБЯЗАТЕЛЬНО на Linux: default trigger = none, без этого панель не покажет меню
trayIcon.setContextMenuTrigger(ContextMenuTrigger.rightClicked);
trayIcon.setVisible(true);
// выход: trayIcon.dispose()
```

### Pattern 4: Close-to-tray (window_manager 0.7.0 legacy)

**Источник: pub.dev/packages/window_manager README, fetched 2026-11-22 (VERIFIED, цитата):**
«`close()`, `destroy()` and `setPreventClose()` behave as before: with `setPreventClose(true)` the title bar's close button and `close()` report `onWindowClose` and leave the window open, and `destroy()` closes it anyway. … `onWindowClose` also reports close requests when prevent-close is off. `destroy()` bypasses the compatibility layer's callback and veto».

```dart
// core/window_shell.dart — обёртка над window_manager/legacy.dart (main.dart:7 precedent)
abstract interface class WindowShell {
  Future<void> preventClose();
  void onCloseRequest(void Function() cb);   // cb = hide, НЕ destroy
  Future<void> hide();
  Future<void> show();
  Future<void> destroy();                    // ТОЛЬКО пункт меню «Выход»
}
```

### Anti-Patterns to Avoid

- **onTrayIconMouseDown для show/hide на Linux** — событие НЕ приходит (см. Pitfall 1); это не баг проекта, это документированное поведение SNI+панель.
- **Забыть setContextMenuTrigger** — default `none`, меню не появится (pub.dev README: «The default context-menu trigger is `none`; setting a menu alone does not enable it»).
- **Потерять ссылку TrayIcon** — «a wrapper that is garbage-collected releases its native handle, and that removes the icon» (pub.dev README).
- **Гадать про de-путь хоткея в рантайме** — комбинации адаптеров статичны, показывать таблицу-справку, не «детектить» DE.
- **Тянуть toml-пакет** — запрещено (CONTEXT:38, Must-NOT 7); hotkey-строка достаётся line-based regex по известному формату.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SNI/D-Bus-трей | свой StatusNotifierItem-клиент | tray_manager 0.9.0 | D-Bus SNI-протокол, меню-экспорт, идентичность Item — целая нативная подсистема уже в nativeapi C++ core |
| Prevent-close/veto окна | свой close-handler на GTK-уровне | window_manager setPreventClose | runner-интеграция + совместимость с legacy-импортами main.dart |
| Извлечение asset в temp-файл | rootBundle.load + File.write | ImageAsset.fromAsset | уже реализовано пакетом: резолвит `<exe>/data/flutter_assets/<name>` (image_asset.dart:32-50) |
| PNG-энкодер «по-взрослому» | decoder/прогрессивные форматы | микро-генератор 80 строк (CRC32+IHDR+IDAT+IEND) | нужны только тривиальные RGBA-заготовки 22×22; dart:io ZLibEncoder делает deflate |
| TOML-парсер | полноценный разбор | regex одной строки `hotkey = "..."` | формат известен и пишется dictad (set_key, config.rs:180-194); нужен ровно один ключ |

**Key insight:** всё сложное (SNI, D-Bus, veto-close) уже в nativeapi-ядре, которое в этом проекте УЖЕ собирается (window_manager привёл cnativeapi 0.6.1 в build-кэш: `app/.dart_tool/hooks_runner/cnativeapi/` с tray_manager_linux.cpp в списке компиляции). Фаза добавляет только Dart-биндинг поверх уже собранного core.

## Runtime State Inventory

Фаза НЕ переименовывает/не мигрирует — но трей вводит новое OS-состояние (SNI-регистрация), фиксирую по категориям:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | Ничего нового: config.toml приложение ТОЛЬКО ЧИТАЕТ (Must-NOT 3); новых персистентных-store нет (история in-memory, README:199) | none |
| Live service config | dictad-сокет/юниты НЕ трогаются; SNI-регистрация приложения — новое, живёт только пока app запущен; Item Id = имя executable (changelog 0.8.0: «It is the GApplication ID or the executable name») — имя бинаря `dicta_app` стабильно | none (проверить UAT: иконка не дублируется после рестарта app) |
| OS-registered state | Ничего: app не ставит autostart/юниты в этой фазе (evdev-hotkey — демона, не приложения) | none |
| Secrets/env vars | DICTA_CONFIG (чтение, mirrors config.rs:174); DICTA_SOCK уже используется (daemon_engine.dart:17) | none |
| Build artifacts | pub-cache уже содержит nativeapi 0.6.1/cnativeapi 0.6.1 (приведён window_manager 0.7.0); после добавления tray_manager pub resolve добавит только Dart-байндиги | `flutter pub get` в задаче-инсталле |

## Common Pitfalls

### Pitfall 1: Клики по иконке трея НЕ приходят на Linux (КРИТИЧНО для TRAY-01)
**What goes wrong:** план рассчитывает на `onTrayIconMouseDown` → show/hide; на Linux событие никогда не приедет — молча.
**Why:** tray_manager 0.9.x на Linux — SNI через D-Bus; клики забирает панель (KDE Plasma) и сама открывает экспортированное меню. Документировано: pub.dev README «Linux still reports no tray icon clicks at all: the panel keeps them and opens the menu itself» и changelog 0.7.0 «Tray icon clicks are **not** reported there». Native события TrayIconClickedEvent/TrayIconRightClickedEvent/TrayIconDoubleClickedEvent — «none of them on Linux».
**How to avoid:** «показать/скрыть» — пункт меню (панель открывает меню по клику ЛЮБОЙ кнопкой); UAT-формулировку TRAY-01 согласовать (см. RESOLVED #1).
**Warning signs:** живой трей есть, клики «не работают» — это не баг, так задумано SNI.

### Pitfall 2: Asset-путь иконки
**What goes wrong:** иконка не грузится (`setIcon` кидает ArgumentError на всех платформах — pub.dev README).
**Why:** `ImageAsset.fromAsset` резолвит путь `<exe_dir>/data/flutter_assets/<name>` (image_asset.dart:32-36, прочитано живьём) — asset обязан быть объявлен в pubspec `flutter.assets:` (pubspec.yaml сейчас БЕЗ assets-секции, строки 65-67) и запрошен по pubspec-имени.
**How to avoid:** объявить `assets/tray/` в pubspec; в debug-сборке путь тоже работает (data/flutter_assets есть в debug-build too). Никакого rootBundle/temp-файла не нужно.
**Warning signs:** ArgumentError при старте трея.

### Pitfall 3: Plugin-каналы в тестах
**What goes wrong:** TrayManagerService создан в widget-тесте → MissingPluginException/тишина каналов.
**Why:** platform-channel не существует в flutter test runner.
**How to avoid:** bootstrap трея ТОЛЬКО из prod-main (Pattern 2); buildFakeApp никогда не override trayServiceProvider реальной реализацией; интерфейс TrayService не импортирует tray_manager. Bonus: nativeapi «Calls a platform has no use for are ignored instead of throwing MissingPluginException» (README) — но полагаться на это нельзя: фейк — канон.
**Warning signs:** тест, который «случайно» создаёт реальный сервис.

### Pitfall 4: Конфиг-парсинг без toml-библиотеки
**What goes wrong:** regex ловит `hotkey` в комментарии/пасте-строке; или ожидается секция `[...]`.
**Why:** живой формат — плоский TOML БЕЗ секций (живой ~/.config/dicta/config.toml: 11 строк, все ключи top-level; config.example.toml:80: `hotkey = "ctrl+alt+v"`), НО в живом конфиге строки hotkey ВООБЩЕ НЕТ → демон на default "off" (config.rs:90). Инлайн-комментарии после `#` есть (config.toml:4,7).
**How to avoid:** парсить построчно: `RegExp(r'^\s*hotkey\s*=\s*"([^"]*)"')` на trimmed-строке; нет матча → "off"; игнорировать `#`-хвост автоматически (регекс до кавычек). Не «дофантазировать» секции. Чистая функция `String parseHotkey(String text)` + fixture-тесты: example-текст → "ctrl+alt+v", живой-подобный → "off".
**Warning signs:** тест только на happy-path.

### Pitfall 5: Кириллица/гейт и «путь иконки» в лаунчере
**What goes wrong:** русские лейблы меню трея написаны литералами в lib → cyrillic_gate; или лейблы в .arb, но меню нативное — а .arb-значение берётся ДО MaterialApp (нет context).
**Why:** нативное меню живёт вне дерева виджетов; AppLocalizations требует context.
**How to avoid:** лейблы меню — .arb-ключи, полученные через `lookupAppLocalizations(const Locale('ru'))` (precedent main.dart:14 — так уже сделано для title окна, ДО runApp); это не литералы, гейт проходит.
**Warning signs:** гейт падает на tray_manager_service.dart.

### Pitfall 6: SNI на GNOME / нестандартных панелях
**What:** «Not Showing in GNOME» — известный issue пакета: GNOME показывает SNI только с расширением AppIndicator (README пакета, раздел Known Issues). KDE Plasma хостит нативно.
**How to avoid:** это окружение пользователя, не код; HK-карточка/DIAG не обязана это чинить, UAT идёт на KDE (машина пользователя — Plasma 6).

## Code Examples

### Close-to-tray: полный шов с тестом

```dart
// core/window_shell.dart
import 'package:window_manager/legacy.dart';   // main.dart:7 precedent

class WindowManagerShell implements WindowShell {
  @override
  Future<void> preventClose() => windowManager.setPreventClose(true);

  @override
  void onCloseRequest(void Function() cb) {
    // legacy WindowListener-паттерн: register listener, onWindowClose → cb()
  }

  @override
  Future<void> hide() => windowManager.hide();
  @override
  Future<void> show() => windowManager.show();
  @override
  Future<void> destroy() => windowManager.destroy();
}

// TRAY-02 unit: фейк + handler
class FakeWindowShell implements WindowShell {
  int hideCount = 0, destroyCount = 0;
  void Function()? _onClose;
  @override
  void onCloseRequest(void Function() cb) => _onClose = cb;
  @override
  Future<void> hide() async => hideCount++;
  @override
  Future<void> destroy() async => destroyCount++;
  // ...
}
test('закрытие окна сворачивает, выход убивает', () {
  final shell = FakeWindowShell();
  wireCloseToTray(shell);          // тестируемая логика: onClose → hide
  shell._onClose!();
  expect(shell.hideCount, 1);
  expect(shell.destroyCount, 0);   // close ≠ exit
  wireMenuExit(shell);              // пункт меню «Выход» → destroy
  // ...
});
```

### parseHotkey — чистая функция (fixture-тестируемая)

```dart
// core/config_path.dart
import 'dart:io';

/// Mirrors dicta-core config_path (config.rs:173-178):
/// DICTA_CONFIG → $XDG_CONFIG_HOME|~/.config /dicta/config.toml.
String resolveConfigPath() {
  final override = Platform.environment['DICTA_CONFIG'];
  if (override != null && override.isNotEmpty) return override;
  final xdg = Platform.environment['XDG_CONFIG_HOME'] ??
      Platform.environment['HOME']!;   // тест-стенд передаёт HOME
  return '$xdg/dicta/config.toml';      // (при XDG_CONFIG_HOME без /dicta см. config.rs:176)
}

/// Плоский TOML без секций, ключ top-level (живой формат сверен 2026-11-22).
/// Нет строки → null → caller показывает daemon default "off" (config.rs:90).
final _hotkeyRe = RegExp(r'^\s*hotkey\s*=\s*"([^"]*)"');

String? parseHotkey(String tomlText) {
  for (final line in tomlText.split('\n')) {
    final m = _hotkeyRe.firstMatch(line);
    if (m != null) return m.group(1);
  }
  return null; // "off" default
}
// Тест: parseHotkey(exampleTomlText) == 'ctrl+alt+v' (config.example.toml:80)
// Тест: parseHotkey(liveTomlText) == null            (живой config без hotkey-строки)
```

### Генератор PNG (tools/gen_tray_icons.dart, чистый dart:io, БЕЗ системных зависимостей)

```dart
// Минимальный валидный PNG: signature + IHDR + IDAT(zlib) + IEND, RGBA 8-bit.
// CRC32 — табличный, ~12 строк; deflate — dart:io ZLibEncoder (zlib-поток, raw: false).
import 'dart:io';
import 'dart:typed_data';

final _crcTable = List<int>.generate(256, (n) {
  var c = n;
  for (var k = 0; k < 8; k++) {
    c = (c & 1) != 0 ? (0xEDB88320 ^ (c >> 1)) : (c >> 1);
  }
  return c;
});

int _crc32(List<int> bytes) {
  var c = 0xFFFFFFFF;
  for (final b in bytes) {
    c = _crcTable[(c ^ b) & 0xFF] ^ (c >> 8);
  }
  return (c ^ 0xFFFFFFFF) & 0xFFFFFFFF;
}

List<int> _chunk(String type, List<int> data) {
  final out = <int>[
    ..._u32(data.length), ...type.codeUnits, ...data, ..._u32(_crc32([...type.codeUnits, ...data])),
  ];
  return out;
}

// IHDR: w h 8 6(RGBA) 0 0 0; scanline: filter-byte 0 + w*4
// pixels: заливка круга (dist<=r) с цветом по иконке:
//   idle: спокойный синий #35689F (brand, main.dart:37 fallback seed)
//   recording: красный
void main() {
  const size = 22, r = 8;
  for (final e in [('idle', [53,104,159,255]), ('recording', [229,57,53,255])]) {
    final rows = <int>[];
    for (var y = 0; y < size; y++) {
      rows.add(0);
      for (var x = 0; x < size; x++) {
        final inside = (x - 10.5) * (x - 10.5) + (y - 10.5) * (y - 10.5) <= r * r;
        rows.addAll(inside ? e.$2 : [0, 0, 0, 0]);
      }
    }
    final ihdr = [..._u32(size), ..._u32(size), 8, 6, 0, 0, 0];
    final png = [
      ...[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
      ..._chunk('IHDR', ihdr),
      ..._chunk('IDAT', ZLibEncoder().convert(rows)),
      ..._chunk('IEND', <int>[]),
    ];
    File('app/assets/tray/${e.$1}.png').createSync(recursive: true);
    File('app/assets/tray/${e.$1}.png').writeAsBytesSync(png);
  }
}
// Запуск: dart run tools/gen_tray_icons.dart; результат КОММИТИТСЯ (binarniki в git —
// как fixture: генератор воспроизводим, ассет не требует сборочного шага).
```

### Маппинг состояния → иконка (тестируемая чистая функция)

```dart
// engine.dart:13 (verbatim, прочитано этой сессией):
// enum DaemonState { idle, recording, transcribing }
TrayIconState iconFor(DaemonState state) => switch (state) {
      DaemonState.recording => TrayIconState.recording,
      _ => TrayIconState.idle,   // idle/transcribing/disconnected — обычная иконка
    };
// TRAY-01 unit: dictaStatusProvider эмитит recording → FakeTrayService.iconCalls == [recording]
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| tray_manager 0.5.x: per-platform plugin-код, libayatana-appindicator | nativeapi C++ core (SNI напрямую, D-Bus) | 0.7.0 (2026); 0.9.0 — Flutter 3.50 | libayatana больше НЕ нужна (README: «`libayatana-appindicator3-dev` is no longer needed»); сборке нужны только libgtk-3/libx11/libxi — все на машине есть (проверено pkg-config/наличием заголовков) |
| 0.5.x API: trayManager singleton + TrayListener | native API: TrayIcon.create()/Menu.create()/events; legacy.dart — deprecated мост | 0.7.0+ | новый код — native API; legacy.dart «will be removed in a later release» |
| window_manager standalone plugin | тоже nativeapi-ядро | 0.7.x | общий C++ core с tray_manager — совместимость гарантирована структурно (один hooks_runner/cnativeapi) |

**Deprecated/outdated:**
- `package:tray_manager/legacy.dart` — deprecated bridge (@Deprecated, will be removed) — НЕ использовать в новом коде.
- tray_manager 0.5.x-сниппеты из блогов (`trayManager.instance.setIcon`) — в 0.9.x такого API нет: `TrayManager.instance` в native API — менеджер иконок, ставится через `TrayIcon.create()`.

## Open Questions (RESOLVED)

1. **Точная интеграция tray_manager 0.9.0 на Linux** — **RESOLVED (с отклонением):** native API `TrayIcon.create()` + `ImageAsset.fromAsset` + `Menu.create()` + `setContextMenuTrigger(rightClicked)` (сниппет Pattern 3, VERIFIED pub.dev README fetched 2026-11-22). `TrayManager.instance.setIcon` НЕ существует в 0.9.x — это 0.5.x-форма. **Отклонение:** `TrayManagerListener.onTrayIconMouseDown` на Linux НЕ приходит (документировано) — «левый клик = показать/скрыть» реализуется пунктом меню; панель KDE сама открывает меню по клику. Совместимость с window_manager 0.7.0 структурная: оба на nativeapi ^0.6.0, общий C++ core уже собирается в app. Версия для pin: **0.9.0** (0.9.1, вышедшая сегодня, — только bump nativeapi 0.6.2; 0.9.0 остаётся валидной и совместимой с разрешённым 0.6.1).
2. **Откуда PNG-иконки** — **RESOLVED:** микро-Dart-генератор `tools/gen_tray_icons.dart` (dart:io ZLibEncoder + CRC32, БЕЗ ImageMagick/inkscape/пакетов; precedent — tools/fetch-model.sh, README:180), артефакты коммитятся в `app/assets/tray/{idle,recording}.png` (22×22 RGBA, 2 иконки достаточно: idle/transcribing/disconnected — обычная, recording — красная). Рантайм-резолв: `ImageAsset.fromAsset('assets/tray/idle.png')` — пакет сам превращает asset в FS-путь `<exe>/data/flutter_assets/<name>` (VERIFIED image_asset.dart:28-51); temp-файл и rootBundle НЕ нужны; asset объявить в pubspec.
3. **Тест-стратегия** — **RESOLVED:** TrayService-шов (Pattern 1, канон hfClientProvider providers.dart:21) + WindowShell-шов для close-to-tray; фейкаются: setIconState-вызовы (список iconCalls), колбэки меню (прямой вызов зарегистрированного cb в тесте), window hide/show/destroy (счётчики); close-to-tray покрыт юнит-уровнем: `onCloseRequest` handler → assert hide без destroy (Pattern 4). Интеграция в тесты: bootstrap трея живёт ТОЛЬКО в prod-main (Pattern 2) — buildFakeApp ничего не знает о tray_manager; фейк подставляется override trayServiceProvider, как hfClient/downloadEngine в fake_dicta_engine.dart:177-183.
4. **Чтение config.toml в тестах** — **RESOLVED:** живой формат — **плоский TOML, БЕЗ секций** (живой ~/.config/dicta/config.toml: 11 строк top-level ключей, инлайн-`#`-комментарии; config.example.toml — то же + строка 80 `hotkey = "ctrl+alt+v"`). В живом конфиге строки `hotkey` НЕТ → демон на default `"off"` (config.rs:49 + :90 verbatim ниже). Парсер: line-based regex `^\s*hotkey\s*=\s*"([^"]*)"` без toml-пакета; `resolveConfigPath` зеркалит config.rs:173-178 (DICTA_CONFIG → XDG config dir/dicta/config.toml); fixture-тесты на текстах example/live-подобных. Что показывать для DE-путей: статичная таблица (KWin Alt+V / Hyprland SUPER,V / Sway $mod+v / GNOME <Ctrl><Alt>v / evdev из config) + текущее evdev-значение из config + предупреждение о рестарте демона.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Смена `trayIcon.icon` в рантайме обновляет иконку в SNI-панели KDE без пересоздания TrayIcon | Standard Stack / TRAY-01 | малая: nativeapi example заявляет «several icons, animated icons» (README); если нет — fallback: destroy+create в TrayManagerService (шов скрывает). Tracer-задача проверит живьём |
| A2 | `ImageAsset.fromAsset` работает в `flutter run` debug-сборке (путь `data/flutter_assets` существует у debug-бинаря тоже) | Pitfall 2 | малая: fallback `Image.fromFile` с явным путём или fromBase64 |
| A3 | KWin-меню открывается по ЛЕВОМУ клику тоже (стандартное поведение Plasma для SNI с меню), не только по правому | Pitfall 1 | малая: меню доступно правым кликом гарантированно (trigger rightClicked); UAT зафиксирует фактическое поведение |
| A4 | pubspec assets-запись `assets/tray/` подтянет новые PNG без перегенерации при сборке | Standard Stack | тривиальная |
| A5 | `lookupAppLocalizations(const Locale('ru'))` доступен для лейблов меню вне build-контекста (precedent main.dart:14 для title) | Pitfall 5 | малая: l10n уже сгенерён, gen-l10n от pubspec:58 |

Все прочие утверждения — VERIFIED/CITED (живой код с file:line, pub.dev fetched 2026-11-22, pub-cache прочитан).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Flutter/Dart SDK | вся фаза | ✓ | 3.50.1 (проверено `flutter --version`) | — |
| libgtk-3-dev | nativeapi Linux build (README пакета) | ✓ | gtk 3.24.52 (pkg-config) | — |
| libx11-dev / libxi-dev | nativeapi Linux build | ✓ | заголовки на месте (/usr/include/X11/Xlib.h; pkg-config xi OK) | — |
| nativeapi / nativeapi_flutter | транзитив tray_manager+window_manager | ✓ | 0.6.1 в pub-cache (проверено ls ~/.pub-cache) | — |
| libayatana-appindicator | — | НЕ нужен | — | nativeapi 0.6.x не требует (README) |
| tray_manager | TRAY | в pub-cache отсутствует (ещё не ставили — ожидаемо) | 0.9.0 на pub.dev (проверено webfetch) | ставится `flutter pub get` после правки pubspec |
| dictad + сокет | ручной UAT/трейсер | ✓ (живой демон пользователя) | — | UAT-пункт |
| KDE Plasma панель (SNI host) | UAT трея | ✓ (машина пользователя — Plasma 6) | — | GNOME требует AppIndicator-расширение (Known Issue пакета) |

**Missing dependencies with no fallback:** нет.
**Missing dependencies with fallback:** tray_manager не в кэше — первый `flutter pub get` фазы его доставит (запланированный шаг инсталляции).

## Validation Architecture

> workflow.nyquist_validation в .planning/config.json отсутствует → секция включена (enabled).

### Test Framework

| Property | Value |
|----------|-------|
| Framework | flutter_test (SDK, pubspec dev_dependencies: flutter_test) |
| Config file | app/pubspec.yaml + default; отдельного конфига нет |
| Quick run command | `cd app && flutter test test/tray_service_test.dart` (один файл) |
| Full suite command | `cd app && flutter test` (25 сьютов сейчас; ~минуты на десктопе) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| TRAY-01 | маппинг DaemonState→иконка; смена иконки по статусу | unit (FakeTrayService: iconCalls) | `cd app && flutter test test/tray_service_test.dart` | ❌ Wave 0 |
| TRAY-01 | пункты меню зовут правильные колбэки (start/stop/cancel/settings/show-hide/exit) | unit (fake cb invocation + FakeDictaEngine) | `cd app && flutter test test/tray_service_test.dart` | ❌ Wave 0 |
| TRAY-01 | меню/иконка живьём на KDE (SNI) | manual UAT | живой трей глазами: клик открывает меню, иконка краснеет при записи | — (UAT-лист) |
| TRAY-02 | onWindowClose → hide, НЕ destroy | unit (FakeWindowShell счётчики) | `cd app && flutter test test/window_shell_test.dart` | ❌ Wave 0 |
| TRAY-02 | пункт «Выход» → destroy | unit + manual UAT (окно закрылось, процесс умер) | там же | ❌ Wave 0 |
| TRAY-02 | close-to-tray живьём (крестик → окно скрылось, app жив) | manual UAT | закрыть окно крестиком → иконка в трее осталась | — (UAT-лист) |
| HK-01 | parseHotkey: example-текст → "ctrl+alt+v"; без строки → null/"off"; инлайн-комментарии | unit + fixtures | `cd app && flutter test test/config_path_test.dart` | ❌ Wave 0 |
| HK-01 | resolveConfigPath: DICTA_CONFIG override → дефолт | unit (Platform.environment паттерн как daemon_engine resolveSocketPath) | там же | ❌ Wave 0 |
| HK-01 | карточка показывает комбинацию, DE-таблицу, предупреждение о рестарте (.arb ru) | widget | `cd app && flutter test test/hotkey_card_test.dart` | ❌ Wave 0 |
| HK-02 | hotkey_manager НЕ в pubspec/lib | grep-гейт | `! grep -rn hotkey_manager app/pubspec.yaml app/lib/` | гейт плана (см. ниже) |
| HK-02 | двойного срабатывания нет при живом демон-хоткее | manual UAT | нажать демон-хоткей при открытом app → одна запись, один stop | — (UAT-лист) |
| UI-наследие фазы 4 (T4) | MergeSemantics/arb/liveRegion/свёртка и т.д. | widget-продолжения существующих сьютов | `cd app && flutter test test/settings_screen_test.dart` (и соседние) | частично ✅ (расширить) |

**HK-02 гейт-команда (верификация плана, не тест-файл):** `grep -rn "hotkey_manager" app/pubspec.yaml app/lib/ app/test/` — пустой вывод = зелёный. Проверено этой сессией: сейчас упоминаний нет (только .dart_tool-шум сборки cnativeapi, не код).

### Sampling Rate

- **Per task commit:** `cd app && flutter test test/<файлы задачи> -x` + `cd app && flutter analyze`
- **Per wave merge:** `cd app && flutter test` (полный сьют)
- **Phase gate:** полный сьют зелёный + гейт кириллицы + grep-гейт HK-02 до `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `app/test/fake_tray_service.dart` — фейк шва (iconCalls, registered callbacks, счётчики show/hide/destroy)
- [ ] `app/test/fixtures/config_example_toml.txt` — текст config.example.toml как fixture (hotkey = "ctrl+alt+v")
- [ ] `app/test/fixtures/config_live_no_hotkey.txt` — живой-подобный конфиг БЕЗ hotkey-строки
- [ ] `app/test/tray_service_test.dart` — TRAY-01 (state→icon, menu callbacks)
- [ ] `app/test/window_shell_test.dart` — TRAY-02
- [ ] `app/test/config_path_test.dart` — HK-01 парсинг/резолв
- [ ] `app/test/hotkey_card_test.dart` — HK-01 карточка
- [ ] `tools/gen_tray_icons.dart` + сгенерированные `app/assets/tray/*.png` (коммит артефакта)

**Manual UAT items (живой трей глазами — в UAT-лист фазы):**
1. Иконка dicta видна в трее KDE сразу после старта app.
2. Клик по иконке открывает меню (левый/правый — зафиксировать фактическое поведение панели).
3. Меню: показать/скрыть окно работает; start/stop/cancel зовут демон (состояние видно на дашборде и в иконке).
4. Иконка переключается на «запись» при старте записи и обратно при idle.
5. Крестик окна: окно скрылось, app жив в трее; запись продолжается.
6. «Выход» в меню: app полностью завершился (проверить процесс).
7. HK-01 карточка: показывает текущее evdev-значение (у живого конфига — «не задан (off)»), таблицу DE-путей, предупреждение о рестарте.
8. Демон-хоткей при живом app: одно срабатывание, без двойного stop.

## Security Domain

Существенная security-поверхность фазы — та же, что фазы 1-4 (unix-сокет к демону); новые векторы:

| ASVS-зона | Applies | Standard Control |
|-----------|---------|-----------------|
| V4 Access Control | нет нового | демон валидирует все команды; app лишь отправитель (не меняется) |
| V5 Input Validation | да (HK-01) | parseHotkey — regex на ПОСТОЯННОМ формате файла, который пишет сам dictad (config.rs:180-194 set_key); отображается, НЕ исполняется; никаких hotkey-строк в команды демона |
| V12 File Handling | да | config.toml — только чтение (Must-NOT 3), пути только из env/известных дефолтов; никаких записей в конфиг |

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Injection через hotkey-строку | Tampering | строка попадает ТОЛЬКО в Text виджета; ни в exec, ни в set-команду (hotkey ∉ EDITABLE_KEYS — смена через протокол невозможна в принципе) |
| Печать OS-ошибок в UI | Information Disclosure | продолжить канон: error.runtimeType (обязательство #2, daemon_engine.dart:61) |

Payload-privacy канон (никаких print/логов текста, Must-NOT 5) распространяется на tray-меню: лейблы статичны, текст диктовок в трей не попадают.

## Sources

### Primary (HIGH confidence)

- **pub.dev/packages/tray_manager** (README + CHANGELOG, fetched 2026-11-22) — native API сниппет, «Linux still reports no tray icon clicks», setContextMenuTrigger-требование, requirements (Flutter 3.50/Dart 3.14, gtk3/x11/xi), legacy-мост deprecated, 0.9.0/0.9.1 changelog, SNI/GNOME Known Issue, tray manager download stats (287k/wk, verified publisher leanflutter.dev)
- **pub.dev/packages/window_manager** (README, fetched 2026-11-22) — setPreventClose/onWindowClose/destroy semantics verbatim, nativeapi-ядро, legacy.dart
- **~/.pub-cache/hosted/pub.dev/nativeapi_flutter-0.6.1/lib/src/widgets/image_asset.dart:28-51** (прочитан живьём) — ImageAsset.fromAsset резолвит `<exe>/data/flutter_assets/<name>` → Image.fromFile
- **~/.pub-cache/hosted/pub.dev/window_manager-0.7.0/pubspec.yaml** (прочитан) — deps `nativeapi: ^0.6.0`, `nativeapi_flutter: ^0.6.0`
- **Живой код репо (Read этой сессии):** config.rs:49,90,142-160,173-178,180-194; config.example.toml:80; ~/.config/dicta/config.toml (весь, 11 строк); README.md:117,125-135,223-234; adapters/README.md (KWin Alt+V / Hyprland SUPER,V / Sway $mod+v / GNOME / evdev); app/pubspec.yaml:22,33-38,55-63; app/lib/main.dart:7,11-23; core/engine.dart:13; core/providers.dart:14-25,57-94; core/daemon_engine.dart:17-22,61; core/hf/hf_client.dart:8-10; features/shell/shell.dart:3,57-78; test/fake_dicta_engine.dart:162-204
- **Окружение (shell этой сессии):** Flutter 3.50.1; gtk3 3.24.52 + x11/xi заголовки; pub-cache nativeapi 0.6.1; hooks_runner cnativeapi-сборка в app/

### Secondary (MEDIUM confidence)

- README.packages «Who's using it» (Airclap, Biyi, scrcpy buddy) — индикатив зрелости, не факт API

### Tertiary (LOW confidence)

- нет использованных

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — версии и API сверены pub.dev (fetched сегодня) + pub-cache живьём
- Architecture: HIGH — швы повторяют живой канон фазы 4 (providers.dart, fake_dicta_engine.dart прочитаны построчно)
- Pitfalls: HIGH — главный pitfall (нет кликов на Linux) — прямая документированная цитата pub.dev README
- Конфиг-формат: HIGH — оба файла прочитаны целиком живьём

**Research date:** 2026-11-22
**Valid until:** 2027-01-22 (стабильная область; pub.dev-версии перепроверить, если план пойдёт позже 30 дней — пакеты leanflutter сегодня (22.11) выпустили патчи 0.9.1/0.7.1, движение есть)
