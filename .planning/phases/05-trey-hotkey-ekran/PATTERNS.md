# Phase 5: Трей + хоткей-экран - Pattern Map

**Mapped:** 2026-11-22
**Files analyzed:** 12 (6 новых, 6 модифицируемых)
**Analogs found:** 9 / 12 (3 позиции — No Analog: plugin-интеграция трея, PNG-ассеты+tool, close-to-tray)

Все названные аналоги — git-tracked (сверено `git ls-files` 16/16; весь
`app/lib`, `app/test`, `app/tool` — tracked, зеркал нет). Гейт единственной
фабрики фейков живой: `grep -c "implements DictaEngine"` → только
`fake_dicta_engine.dart` (== 1). Кириллица в `app/lib/**/*.dart` вне
`lib/l10n/generated/` запрещена гейтом `app/tool/cyrillic_gate.sh` — русский
UI только через `.arb`; протокольные/пользовательские строки в коде —
Unicode-escape (канон `daemon_engine.dart:25-37`).

**Вход исследователя:** RESEARCH.md в каталоге фазы ЕЩЁ НЕТ (на момент
мэппинга только CONTEXT.md). Открытые вопросы CONTEXT:33-38 (точный API
tray_manager 0.9.0 на Linux, резолв пути иконки из assets, источник PNG,
тест-стратегия close-to-tray) НЕ разрешены здесь — планёр обязан
дождаться пометок RESOLVED от researcher. Ниже даны структурные аналоги,
не версионные детали API.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `app/lib/core/tray.dart` (new: TrayService interface + TrayManagerService) | service (seam) | event-driven (SNI-события → меню/иконка) + request-response (menu → start/stop/cancel) | `core/hf/hf_client.dart:8-41` (interface + exception + impl) + `core/hf/download_engine.dart:60-80` (cancel-gate) | exact (роль+поток; plugin-часть — No Analog) |
| `app/lib/core/config_path.dart` (new: resolveConfigPath + extractHotkey) | utility (чистые функции) | transform (текст → путь/hotkey) | `daemon_engine.dart:16-23` (resolveSocketPath) + `engine.dart:93-105` (чистые build*Line) | exact (роль+поток) |
| `app/lib/features/settings/hotkey_card.dart` (new: HK-карточка) | component | file-I/O (read-only config.toml) | `settings_screen.dart:152-160` (_group) + `:264-291` (_field) + `:297-325` (_fetchErrorCard) + `:598-607` (confStatus liveRegion) | exact (роль; чтение файла — частичный, см. No Analog) |
| `app/assets/tray/*.png` + `app/tool/tray_icons.dart` (new: генерация) | config (asset) + tool | — | No Analog (в pubspec НЕТ assets-секции, pubspec.yaml:65-67 закомментированный шаблон; process-аналог tool-скриптов: `tool/status_smoke.dart`, `tool/diag_smoke.dart`) | none |
| `app/pubspec.yaml` (modify: tray_manager 0.9.0 + assets) | config | — | сам файл :30-38 (пакеты с точечной версией: flutter_riverpod 3.5.0, window_manager 0.7.0) | exact |
| `app/lib/core/providers.dart` (modify: trayService/window seam-провайдеры) | store | event-driven | сам файл :20-25 (hfClientProvider/downloadEngineProvider — канон регистрации шва) | exact |
| `app/lib/main.dart` (modify: tray-инициализация + close-to-tray) | config | event-driven | сам файл :9-24 (windowManager init — точка входа); close-to-tray — No Analog | role-match |
| `app/lib/features/settings/settings_screen.dart` (modify: HK-группа в систему) | component | — | сам файл :574-596 (_group(l10n.confGroupSystem)) | exact |
| `app/lib/features/shell/shell.dart` (modify ТОЛЬКО если трей-меню «настройки» переключает вкладку) | component | — | сам файл :23 (локальный `_selected`), :82-89 (IndexedStack) | exact |
| `app/test/fake_tray_service.dart` (new) + `app/test/tray_test.dart` (new) | test | event-driven | `fake_download_engine.dart:11-83` (scenario-enum + gate + счётчик calls) + `fake_dicta_engine.dart:162-204` (buildFakeApp) | exact |
| `app/test/config_path_test.dart` (new) | test | transform | `daemon_engine_framing_test.dart:26-28` (точная wire-форма) + `hf_urls_test.dart:1-15` (verbatim-паритет) | exact |
| `app/lib/l10n/app_ru.arb` + generated (modify: семейства `tray*`/`hk*`) | config (l10n) | — | сам файл: `dash*` :59-118, `hist*` :119-150, `conf*` :235-363, `model*` :365-548 | exact |
| UI-наследие фазы 4 одним задачом (modify по мере касания) | refactor | — | `settings_screen.dart:181-211` (MergeSemantics-слайдер), `models_screen.dart:54-56,120-129` (snackbar/локализация ошибок), `history_screen.dart` (сворачивание — No Analog) | exact / partial |

## Pattern Assignments

### `app/lib/core/tray.dart` (service-seam, event-driven + request-response)

**Analog 1 (шов «interface + impl в одном core-файле»):**
`app/lib/core/hf/hf_client.dart:8-41`

```dart
abstract interface class HfClient {
  Future<String> fetchText(String url);
}

/// Error at the seam: carries the status CODE only, never the response body
/// (payload privacy, daemon_engine.dart:57-58 canon).
class HfHttpException implements Exception { ... }

class HttpHfClient implements HfClient { ... }
```

Копировать СТРУКТУРУ файла: `library;`-шапка с док-комментом «transport
lives only in core/; features/ talk to the abstract interface»
(hf_client.dart:1-3), `abstract interface class TrayService` (методы по
TRAY-01: setIcon(state), setMenu, show/hide/destroy window, listener
click/menu-select), исключение-как-данные с кодом/типом, реализация
`TrayManagerService implements TrayService` — ЕДИНСТВЕННОЕ место, знающее
`tray_manager` (зеркало правила APP-03, как `HttpHfClient` — единственное
место, знающее HttpClient).

**Analog 2 (cancel/intent-gate для асимметрии жизненного цикла):**
`core/hf/download_engine.dart:60-80` (`DownloadCancelGate`) — если события
трея могут прийти до готовности окна/иконки, держать intent-флаг, а не
полагаться на null-поля. Не обязателен, но это единственный в кодовой базе
паттерн «событие раньше объекта».

**Analog 3 (потребитель команд — НЕ трей сам):** команды меню идут через
СУЩЕСТВУЮЩИЙ `DictaControlNotifier` (`dicta_control.dart:68-157`:
start/stop/cancel с `commandInFlight`-сериализацией и errors-as-data),
состояние иконки — `ref.watch(dictaStatusProvider)` (providers.dart:57-73,
100 мс полл, уже живой). tray.dart НЕ зовёт `engineProvider` напрямую и
НЕ заводит свой полл-цикл.

**Регистрация:** `providers.dart:20-25` — точный канон:

```dart
/// HF transport seam - overridden by FakeHfClient in widget tests.
final hfClientProvider = Provider<HfClient>((_) => HttpHfClient());
```

Новые: `trayServiceProvider` (+ при необходимости `windowServiceProvider`)
по той же однострочной форме, с комментарием «overridden by
FakeTrayService in tests».

---

### `app/lib/core/config_path.dart` (utility, transform)

**Analog 1 (env-override → системный путь, чистая функция):**
`daemon_engine.dart:16-23`

```dart
/// DICTA_SOCK (test override) -> XDG_RUNTIME_DIR -> /run/user/`uid`.
String resolveSocketPath() {
  final override = Platform.environment['DICTA_SOCK'];
  if (override != null && override.isNotEmpty) return override;
  final xdg = Platform.environment['XDG_RUNTIME_DIR'];
  ...
}
```

Поведенческий паритет — `dicta-core/src/config.rs:173-178`
(READ-ONLY эталон поведения):

```rust
pub fn config_path() -> PathBuf {
    std::env::var_os("DICTA_CONFIG")
        .map(PathBuf::from)
        .or_else(|| dirs::config_dir().map(|dir| dir.join("dicta").join("config.toml")))
        .unwrap_or_else(|| PathBuf::from("dicta-config.toml"))
}
```

Dart-версия: `DICTA_CONFIG` → `XDG_CONFIG_HOME` → `~/.config` →
`/dicta/config.toml`; последний Resort `"dicta-config.toml"` — сохранить
честно (паритет). Чистая top-level функция, БЕЗ класса и БЕЗ dart:io
File-операций (паттерн engine.dart:93-105 — юнит-тестируемость без IO);
`Platform.environment` уже dart:io, но без сети/файлов — как у
resolveSocketPath.

**Analog 2 (extractHotkey — минимальный строковый поиск):** конфиг ПЛОСКИЙ,
без секций (config.rs:8-72 — все ключи верхнего уровня; `hotkey: String`
:49, default `"off"` :90; dictad пишет строку `hotkey = "…"` построчно,
config.rs:180-192 `set_key` — строковый replace/append, НЕ TOML-серилизатор
секций). Формат строки известен и стабилен:

```dart
/// Flat config.toml line: hotkey = "off"  (dicta-core config.rs:49,90).
/// Minimal regex, NO toml package (a new dependency is forbidden).
String? extractHotkey(String configText) =>
    RegExp(r'^\s*hotkey\s*=\s*"([^"]*)"\s*$', multiLine: true)
        .firstMatch(configText)? // точный якорь строки — как set_key
        ?.group(1);
```

Мусор/отсутствие ключа → `null` → карточка честно показывает «off»
(дефолт демона) или «не найдено» — толерантность канона
`DictaStatus.fromReplyLine` (engine.dart:35-54): битый ввод = данные, не
исключение.

**Чтение файла** — отдельно от чистых функций, в карточке или FutureProvider
(`dart:io File.readAsStringString` → errors-as-data), по образцу
`settingsSnapshotProvider` (providers.dart:41-53): fetch-on-demand, без
поллинга, ошибка → AsyncError, не дефолты.

**Важно (дивергенция путей):** снимок настроек УЖЕ несёт путь конфига
демона — `config_path` в ответе settings (`settings.dart:110`,
поле `configPath` :153; рендерится read-only полем
`confConfigPathField`, settings_screen.dart:590-595). HK-карточка обязана
ПРЕДПОЧИТАТЬ `s.configPath` из `settingsSnapshotProvider`, когда он
непустой; `resolveConfigPath()` — чистый fallback для disconnected-демона
и для юнит-тестов. Планировщик зафиксировать это в задаче.

---

### `app/lib/features/settings/hotkey_card.dart` (component)

**Analog 1 (группа-карточка в существующем экране):**
`settings_screen.dart:152-160` — `_group(title, children)` (titleMedium,
SizedBox-ритм 12/8); место врезки — СУЩЕСТВУЮЩАЯ группа «система»
(`_group(l10n.confGroupSystem, …)` :574-596), НОВЫЙ rail-пункт не вводить
(CONTEXT must-NOT 8; rail = 4 пункта, shell.dart:57-78).

**Analog 2 (read-only поле):** `_field` settings_screen.dart:264-291 —
MergeSemantics + Semantics(label:) вокруг TextField, `Key('hk…')` по
канону `confConfigPathField`. Значение хоткея — read-only текст
(редактирование вне приложения, CONTEXT must-NOT 3).

**Analog 3 (честные состояния загрузки/ошибки):**
- loading: `_fetchLoading` :328-345 (спиннер, не дефолт-форма);
- ошибка чтения config.toml: `_fetchErrorCard` :297-325 (Card + retry
  OutlinedButton + `ref.invalidate`), уже скопирован с models-screen
  WR-06 — третий call-site разрешён, это канон честных transport-состояний;
- verbatim-значения (путь evdev/DE, `hotkey = "off"`) — по канону
  verbatim-демон-текста last_text_card (фаза 3 PATTERNS, :91-110):
  показывать как есть, не выдумывать.

**Analog 4 (liveRegion-строка статуса):** confStatus
settings_screen.dart:598-607 — `Semantics(liveRegion: true)` вокруг
результата чтения; предупреждение HK-01 «применится после рестарта
демона» — статичный .arb-текст рядом (не liveRegion, он не transient).

**Analog 5 (тексты DE-пути):** комбинации адаптеров статичны и известны
(CONTEXT:19: KWin `Alt+V`, Hyprland/Sway include, GNOME
`<Ctrl><Alt>v` → `dictactl toggle`) — оформлять как .arb-константы
семейства `hk*`, НЕ как код-строки (кириллица-гейт). Помечать источник
(evdev `hotkey` vs DE-скрипт) честно, по REQUIREMENTS HK-01.

---

### Wire-up: `main.dart` / `shell.dart` / фейки

**`main.dart` (modify):** :9-24 — единственное место, где живёт
window_manager (legacy-импорт :7, ensureInitialized :11,
waitUntilReadyToShow :18-21). Трей-инициализация встаёт СЮДА: после
`windowManager.ensureInitialized()` — создать TrayManagerService через
`ProviderContainer` (или передать через ProviderScope.overrides),
зарегистрировать close-to-tray (`setPreventClose(true)` +
`onWindowClose → hide()`), честный выход — только пункт меню
(`windowManager.destroy()`). Тесты main() НЕ запускают — plugin-каналы
в тестах недостижимы уже по построению; ВТОРОЙ гейт: buildFakeApp/buildTestApp
не должны получать настоящего TrayService.

**Регистрация шва для тестов:**
- `fake_dicta_engine.dart:162-204` `buildFakeApp` — overrides-список
  :177-183; канон «null means do NOT override» :166-169. Новые параметрыры
  `TrayService? tray` (+ WindowService?) по тому же правилу: suites, не
  касающиеся трея, остаются plugin-free. ИЛИ (дешевле): провайдер по
  умолчанию возвращает no-op/undан摹 fake, если планёр решит так — но
  аналог-канон — явный override-параметр.
- `shell_test.dart:14-31` `buildTestApp` — suite-local scaffold; если
  сьют трея строит свой scaffold, регистрировать `AppTextStyles.from`
  (:23) обязательно (гейт mutedOf-assert, theme_ext.dart:31-38).

**`shell.dart` (modify — ТОЛЬКО если меню «настройки» переключает вкладку):**
выбор вкладки — приватный локальный стейт `_ShellWidgetState._selected`
(shell.dart:23), провайдера НЕТ. Трей-меню «настройки» = показать окно +
выбрать вкладку 3 → нужен меж-(main↔shell) сигнал: либо выставить
`_selected` через новый простой Notifier (аналог
`HistoryQueryNotifier`, history.dart:60-68 — notifier+provider в одном
файле, начальное значение и всё), либо ограничиться «показать окно» без
переключения. Решение за планёром; без обоснования НЕ менять
NavigationRail (:52-79).

**Меню трея (TRAY-01):** start/stop/cancel/настройки/выход — команды
ЧЕРЕЗ `ref.read(dictaControlProvider.notifier).{start,stop,cancel}()`
(dicta_control.dart:68-157; commandInFlight-сериализация уже встроена),
«настройки» → окно+вкладка (см. выше), «выход» → destroy. Никаких новых
команд протокола (заморозка).

---

### Тесты

**`app/test/config_path_test.dart` (new):** канон verbatim-паритета
`hf_urls_test.dart:1-15` («strings copied verbatim from the frozen Rust
reference») и точной wire-формы `daemon_engine_framing_test.dart:26-28`:

```dart
test('resolveConfigPath mirrors dicta-core config.rs:173-178', () { ... });
// DICTA_CONFIG override / XDG_CONFIG_HOME / ~/.config fallback / last resort
test('extractHotkey finds the flat line and tolerates garbage', () {
  expect(extractHotkey('hotkey = "alt+v"\n'), 'alt+v');
  expect(extractHotkey('hotkey = "off"'), 'off');
  expect(extractHotkey('[audio]\nhotkey="x"'), 'x'); // ключи плоские; секций нет —
  // строка в секции всё равно валидна для построчного поиска (как set_key)
  expect(extractHotkey(''), isNull);
  expect(extractHotkey('hotkey = off'), isNull); // без кавычек — мусор
});
```

env-переопределения — `Platform.environment` в тесте не мутабелен:
`resolveConfigPath` проектировать с injectable-параметром
(`resolveConfigPath({String? configOverride, String? xdg})`) или
принимать map — как `DaemonEngine([String? socketPath])`
(daemon_engine.dart:40-41) принимает уже разрешённый путь.

**`app/test/fake_tray_service.dart` (new):** канон `fake_download_engine.dart`
— ВТОРОЙ seam-fake отдельным файлом (не клонировать FakeDictaEngine):
scenario-поле, `final List<…> calls`-счётчик (:23), `Completer<void>? gate`
(:27), «sync writes keep the fake on the microtask queue» (:60-61). Гейт
«одна фабрика на шов»: `implements TrayService` == 1.

**`app/test/tray_test.dart` (new):** каркас `download_state_test.dart:34-58`
— `ProviderContainer(overrides: [trayServiceProvider.overrideWithValue(fake),
engineProvider.overrideWithValue(FakeDictaEngine…)])`,
`addTearDown(container.dispose)`, settle-цикл; close-to-tray логика
(setPreventClose + onWindowClose → hide) — юнит-уровнем через fake:
эмитировать close-событие → ассерт hide-вызова, ассерт «destroy НЕ вызван»;
меню start/stop/cancel → ассерт `fake.setCalls`/стейт-переходы через
DictaControlNotifier; иконка: `dictaStatusProvider` state → ассерт
`setIcon` вызван с нужным состоянием. Widget-сьюты трея (если нужны) —
через buildFakeApp с подменённым tray.

**Существующие сьюты не должны сломаться:** shell_test.dart (:34-48
проверяет rail) остаётся зелёным при неизменном rail; grep
`placeholderScreen`/`confConfigPathField` перед правками settings_screen.

---

### `app/lib/l10n/app_ru.arb` + generated (modify)

Канон: `"key": "текст"` + `"@key": {"description": …}`; префиксные
семейства. Новые:
- `tray*` — пункты меню (trayStart/trayStop/trayCancel/traySettings/
  trayExit), tooltip;
- `hk*` — карточка: hkTitle, hkCurrentValueLabel, hkUnavailable
  (файл не читается), hkRestartWarning («применится после рестарта
  демона»), hkEvdevPath/hkDePath-подписи, hkDeKwin/hkDeHyprland/hkDeGnome
  (комбинации адаптеров), hkEditHint («правка — только в config.toml,
  вне приложения»).
Snackbar-подтверждения трей-действий — по канону `dashCopied` :91-94 /
`modelDownloadDone` :530-533: transient, только по факту успеха.
Placeholders-объявление — канон `levelSeconds` :44-51. После правки —
`flutter gen-l10n`; generated коммитится, руками не правится.

### UI-наследие фазы 4 (одним задачом, T4)

Аналоги уже в живом коде (фаза 4 их частично погасила — сверить
04-UI-REVIEW перед задачей): MergeSemantics слайдер/сегмент —
settings_screen.dart:181-211, :238-262; snackbar-локализация ошибок —
models_screen.dart:54-56,120-129 (канон уже есть); liveRegion на
fetch-ошибках — confStatus :598-607 и `_fetchErrorCard` :297-325;
сворачивание длинных записей (history) — No Analog (расширяемых строк
в кодовой базе нет; стандартный ExpansionTile/AnimatedCrossFade — планёр
берёт Flutter-канон). daemon_engine.dart:60-62 — `$error` →
`error.runtimeType` (одна строка, payload-privacy; CONTEXT:30).

## Shared Patterns

### Регистрация seam-провайдера
**Source:** `providers.dart:20-25` (hfClientProvider/downloadEngineProvider
— одна строка + doc-коммент «overridden by Fake… in tests»).
**Apply to:** trayServiceProvider (+ window seam). Один шов — один провайдер
— один фейк-файл.

### Errors-as-data
**Source:** `daemon_engine.dart:77-93` + `dicta_control.dart:80-86`
(`catch (_) → genericFailure`); толерантный парс — engine.dart:35-54.
**Apply to:** tray.dart (события/команды никогда не бросают в UI),
config-чтение (битый файл → честная карточка-ошибка).

### Payload privacy
**Source:** `daemon_engine.dart:57-58` (тип+путь, не содержимое);
нарушение `daemon_engine.dart:61` гасим в этой фазе (CONTEXT:30).
**Apply to:** tray-логи/ошибки — только runtimeType/путь иконки, не
пейлоады; никаких print в lib.

### Cyrillic gate
**Source:** `.arb` для UI; `daemon_engine.dart:25-37` (Unicode-escape)
для строковых констант в коде.
**Apply to:** все новые файлы; `bash app/tool/cyrillic_gate.sh` в гейте.

### Plugin-каналы вне тестов
**Source:** `main.dart:9-24` (единственный window_manager-call-site) +
`fake_dicta_engine.dart:166-183` (null-means-no-override) +
`shell_test.dart:14-31`.
**Apply to:** трей инициализируется ТОЛЬКО из main(); тесты получают
FakeTrayService через override; buildFakeApp/buildTestApp остаются
plugin-free (CONTEXT:31).

### Fake-канон (scenario + gate + счётчик)
**Source:** `fake_download_engine.dart:11-83` (enum-сценарии, calls,
downloadGate-Completer, sync-пишущий поток для fake-async);
гейты-предшественники `fake_dicta_engine.dart:39-42,77`.
**Apply to:** FakeTrayService (сценарии кликов/закрытия окна, счётчик
setIcon/hide/destroy, gate для «close в полёте»).

### mounted/owned-guards после await
**Source:** `settings_screen.dart:86,123`; фаза 4 PATTERNS
(calibration_card.dart:81,93).
**Apply to:** асинхронные обработчики HK-карточки (чтение файла).

### AppTextStyles-регистрация (3+1 точки)
**Source:** `theme_ext.dart:3-6` (grep-комментарий), main.dart:46-54,
fake_dicta_engine.dart:190-195, shell_test.dart:21-24.
**Apply to:** любые новые suite-local scaffolds трей-тестов.

## Запреты-по-аналогии

1. **Глобальный хоткей в приложении** — НИКАКИХ hotkey-пакетов и
   register-вызовов на Linux (HK-02, гонка двойного stop); единственный
   источник комбинации — текст `hotkey` из config.toml, и то только
   для ПОКАЗА.
2. **Второй `implements DictaEngine`** — гейт ==1 (fake_dicta_engine.dart:19);
   FakeTrayService — отдельный шов, отдельный файл, `implements
   TrayService` == 1 (канон fake_download_engine.dart:11).
3. **Протокол/демон/Rust** — байт-в-байт к phase-base; меню трея жуёт
   ТОЛЬКО существующие start/stop/cancel/status/set; `hotkey` НЕ в
   EDITABLE_KEYS (config.rs:168-170) — в set-команды не отправлять вовсе.
4. **Запись config.toml** — только чтение (`File.readAsString`);
   TOML-пакет не тянуть (regex/строковый поиск, паритет set_key
   config.rs:180-192); правка хоткея — инструкция на карточке, не код.
5. **dicta-settings crate** — не трогать (декомиссия фаза 8).
6. **Кириллица в lib/ вне generated** — только .arb; escape-паттерн для
   НЕ-user-visible констант (daemon_engine.dart:25-37).
7. **Новые пакеты** — только tray_manager, версия точечно в pubspec
   (pubspec.yaml:30-38 — канон `tray_manager: 0.9.0`); DE-комбинации и
   значения — .arb, не код.
8. **Пятый rail-пункт** — rail закреплён (shell.dart:57-78, 4 пункта,
   фаза 1); HK-карточка — вкладка Настройки; NavigationRail не трогать.
9. **Plugin-каналы в тестах** — TrayManagerService/windowManager создаются
   только в main.dart; buildFakeApp (:162-204) / buildTestApp
   (shell_test.dart:14-31) не должны видеть настоящих сервисов —
   override-канон «null = не подменять» (:166-169).
10. **Второй полл-цикл** — иконка/меню трея читают ТОЛЬКО существующий
    `dictaStatusProvider` (providers.dart:57-73) и `dictaControlProvider`;
    своих таймеров и подписок на сокет в tray.dart не заводить. Окно
    убивается ТОЛЬКО пунктом меню (destroy), close = hide (TRAY-02).

## No Analog Found

| Артефакт | Почему нет аналога | поведенческий источник |
|---|---|---|
| Интеграция tray_manager 0.9.0 (SNI): setIcon из asset, меню, listener | Единственный plugin в проекте — window_manager, и только init-блок (main.dart:9-24); ни одного event-listener-плагина в lib/ нет | RESEARCH (открытый вопрос CONTEXT:35); REQUIREMENTS TRAY-01/02 |
| Close-to-tray: setPreventClose + onWindowClose → hide | window_manager используется лишь при старте; close-handler-формы нет | CONTEXT:21; TRAY-02 |
| PNG-ассеты + tool-генерация валидных PNG байтами | pubspec БЕЗ assets-секции (pubspec.yaml:65-67 — шаблон в комментариях); tool/ — только dart/smoke-скрипты (process-аналог: `tool/status_smoke.dart`) | CONTEXT:24,36 (без ImageMagick; микро-скрипт tool/) |
| Сворачивание длинных записей (UI-наследие е) | grep Expansion\|AnimatedCrossFade по lib/ пуст | Flutter-каноны; 04-UI-REVIEW |

Родственные, но НЕ идентичные: confirm-диалог из No-Analog фазы 4 ТЕПЕРЬ
ЖИВ (`settings_screen.dart:103-124` AlertDialog) — переиспользовать
(тексты конфирма при необходимости); `_fetchErrorCard` (:297-325) —
третий call-site допустим как канон WR-06.

## Предупреждения о расхождениях (для планёра)

1. **RESEARCH.md отсутствует** — открытые вопросы CONTEXT:33-38 не закрыты;
   задачи Wave 1-2 ставить ПОСЛЕ researcher-разрешений (особенно путь
   иконки из assets → temp и API tray_manager 0.9.0 на Linux/Flutter 3.50).
2. **Путь конфига:** HK-карточка предпочитает `s.configPath` из живого
   снимка (settings.dart:110,153 — демон уже рапортует путь); чистый
   `resolveConfigPath` — fallback/тесты; паритет config.rs:173-178
   (включая последний resort `"dicta-config.toml"` и XDG_CONFIG_HOME).
3. **Переключение вкладки из трея:** `_selected` — приватный локальный
   стейт (shell.dart:23); нужен новый маленький Notifier (канон
   history.dart:60-68) или отказ от переключения — решение зафиксировать
   в задаче, чтобы не разморозить rail.
4. **Форма ответа демона фиксирована** (CONTEXT:11) — трей не добавляет
   полей в статус; disconnected-иконка = `status.connected == false`
   (как баннер shell.dart:34-48).
5. **Иконки:** 2-3 PNG (idle/recording[/transcribing]) — количество
   зафиксировать после RESEARCH; тесты иконок — через FakeTrayService
   (assert setIcon-аргумента), НЕ через реальные asset-байты.

## Metadata

**Analog search scope:** `app/lib/**`, `app/test/**`, `app/tool/**`,
`app/pubspec.yaml`, `crates/dicta-core/src/{config,hotkey}.rs` (READ-ONLY
паритет), `.planning/phases/0{2,3,4}-*/PATTERNS.md` (формат),
`.planning/{ROADMAP,REQUIREMENTS}.md` (TRAY/HK).
**Grep-проверки:** `implements DictaEngine` (==1), `assets:` в pubspec
(нет), `Expansion|AnimatedCrossFade|showDialog|TrayManager|hotkey` по
lib/; `git ls-files` 16/16 tracked, зеркал нет.
**Pattern extraction date:** 2026-11-22.
