---
phase: 05-trey-hotkey-ekran
reviewed: 2026-11-22T16:19:18Z
depth: deep
files_reviewed: 33
files_reviewed_list:
  - app/lib/core/tray.dart
  - app/lib/core/window_shell.dart
  - app/lib/core/config_path.dart
  - app/lib/core/providers.dart
  - app/lib/core/daemon_engine.dart
  - app/lib/tray_bootstrap.dart
  - app/lib/main.dart
  - app/lib/features/shell/shell.dart
  - app/lib/features/settings/settings_screen.dart
  - app/lib/features/settings/hotkey_card.dart
  - app/lib/features/history/history_screen.dart
  - app/lib/features/models/hf_browser.dart
  - app/lib/features/models/model_store.dart
  - app/lib/features/models/models_screen.dart
  - app/lib/l10n/app_ru.arb
  - app/lib/l10n/generated/app_localizations.dart
  - app/lib/l10n/generated/app_localizations_ru.dart
  - app/pubspec.yaml
  - app/pubspec.lock
  - app/assets/tray/idle.png
  - app/assets/tray/recording.png
  - tools/gen_tray_icons.dart
  - app/test/fake_tray_service.dart
  - app/test/fake_window_shell.dart
  - app/test/fake_dicta_engine.dart
  - app/test/tray_service_test.dart
  - app/test/window_shell_test.dart
  - app/test/config_path_test.dart
  - app/test/hotkey_card_test.dart
  - app/test/shell_test.dart
  - app/test/history_screen_test.dart
  - app/test/models_screen_test.dart
  - app/test/models_browser_test.dart
findings:
  critical: 0
  warning: 4
  info: 5
  total: 9
status: issues_found
---

# Phase 5: Code Review Report

**Reviewed:** 2026-11-22T16:19:18Z
**Depth:** deep (живые пробы + cross-file трейсинг)
**Files Reviewed:** 33 (app/ дифф b81178b..HEAD + tools/gen_tray_icons.dart)
**Status:** issues_found

## Summary

Ревью фазы 5 «Трей + хоткей-экран» каноном фаз 2-4: не вера в SUMMARY, а живые
пробы. Контрактная база цела и все автоматические гейты Task 6 зелёные одним
прогоном (таблица ниже). Шов изолирован чисто: `tray_manager` живёт только в
`core/tray.dart` (lib == 1, test == 0), `window_manager` — только в
`core/window_shell.dart`; bootstrap только в prod-main; гард иконки по переходам
работает и покрыт тестом (в т.ч. disconnected-ветка без лишнего вызова); крестик =
hide, destroy — только из пункта меню; конфиг только читается (writeAs/openWrite
== 0); UI-наследие фазы 4 закрыто (все три литерал-гейта [HEAD=0 EXPECT==0]
зелёные, liveRegion/excludeSemantics на местах, свёртка >160 с сохранённым
SelectableText).

Найдено 0 BLOCKER, 4 WARNING, 5 INFO. Ключевые: (1) трей-меню обходит
commandInFlight-сериализацию — двойной клик по пункту шлёт повторную команду, а
ошибка ответа демона в трее тихая; (2) невозможность создать SNI-иконку/меню
валит весь старт приложения вместо деградации в windowed-режим (README при этом
обещает «GNOME просто не покажет иконку»); (3) живая смена иконки (A1) так и не
проверена против настоящей панели — smoke Task 2 честно зафиксировал «не
переключена», но после Task 3 повторного live-smoke не было, и запланированный
fallback destroy+create не реализован и не опровергнут; (4) extractHotkey
понимает только двойные кавычки — валидная TOML-строка `hotkey = 'alt+v'`
(literal string, рука пользователя — единственный источник этой строки, dictad
её не пишет) отобразится как «не задан (off)», т.е. карточка солжёт о конфиге
демона.

## Live gates (замерены этим ревью, не скопированы из SUMMARY)

| Gate | Result |
|---|---|
| `cd app && flutter test` | 170/170 passed |
| `flutter analyze` | No issues found! |
| `bash tool/cyrillic_gate.sh` | 0 |
| `dart run tool/status_smoke.dart` | SMOKE OK state=idle, exit 0 |
| `git diff --quiet b81178b HEAD -- crates/ adapters/ systemd/` | RUST-FROZEN |
| implements DictaEngine / TrayService / WindowShell | 1 / 1 / 1 |
| `hotkey_manager` в pubspec+lock+lib | 0 (HK-02) |
| `tray_manager: 0.9.0` pin; lock-diff | 1; только tray_manager-цепочка |
| `dart tools/gen_tray_icons.dart` + `git diff --quiet -- app/assets/tray/` | ICONS-DETERMINISTIC |
| tray_manager в lib/ / test/ | 1 / 0 |
| `$error` в daemon_engine.dart (теперь `error.runtimeType`, :61) | 0 |
| Timer/Stopwatch/Socket.connect в tray-коде | 0 |
| 24 .arb-якоря | 24 |
| README «## Tray (dicta-app)» / «## Трей (dicta-app)» | 1 / 1 |
| Литерал-гейты Task 5: `HTTP ${error.statusCode}`, `delete failed`, `} GB`/`} MB` | 0 / 0 / 0 |
| writeAs/openWrite/FileMode в config_path + hotkey_card | 0 / 0 |
| print( в lib/ | 0 |

## Narrative Findings (AI reviewer)

### WR-01: Трей-меню обходит commandInFlight-сериализацию (двойной клик = повторная команда, ошибка тихая)

**Severity:** WARNING
**File:** `app/lib/tray_bootstrap.dart:58-63` (против `app/lib/features/dashboard/dicta_control.dart:68-87`, `app/lib/features/dashboard/dashboard_screen.dart:32-35`)
**Issue:** План опирается на истину «commandInFlight-сериализация уже жива,
dicta_control.dart:64-157». Живая проверка показывает: сериализация — это только
UI-гейтинг. `DictaControlNotifier.start()/stop()/cancel()` не имеют reentrancy-гарда
(`dicta_control.dart:69` пишет флаг, но не проверяет его на входе); от двойного
вызова защищают лишь кнопки дашборда (`!busy`, dashboard_screen.dart:33-35) и кнопка
inject истории (history_screen.dart:152). Нативное DBus-меню — четвёртый,
единственный незащищённый путь: двойной клик по «остановить» (или «начать запись»)
посылает две команды демону — вторая придёт в-flight, демон ответит ошибкой, и
`lastError` оседает в состоянии, которое из трея никто не показывает: провал в трее
тихий (на дашборде он бы отобразился). Поведенческого повреждения нет (демон
идемпотентен, ошибки — данные), но заявленная планом сериализация в
трей-пути фактически отсутствует, а тест `tray_service_test.dart:82-89` кликает
каждый пункт ровно один раз и гонку не ловит.
**Fix:** Гард в TrayController (меню должно жить по тем же правилам, что и кнопки):

```dart
void _command(Future<void> Function(DictaControlNotifier) run) {
  if (_container.read(dictaControlProvider).commandInFlight) return;
  run(_container.read(dictaControlProvider.notifier));
}
// onMenuStart: () => _command((n) => n.start()), и т.д.
```

(или reentrancy-гард внутри самих методов нотифера — тогда защищены все пути).

### WR-02: Провал создания SNI-иконки/меню валит весь старт приложения вместо деградации

**Severity:** WARNING
**File:** `app/lib/core/tray.dart:63-64,76-77` + `app/lib/main.dart:28-35`
**Issue:** `TrayIcon.create()` или `Menu.create()`, вернувшие null (нет SNI-хоста:
GNOME без AppIndicator; панель ещё не поднята при автозапуске сессии), бросают
`StateError` внутри `await TrayController(...).start()` в `main()` — ДО `runApp`:
окно приложения вообще не открывается. При этом README (секция «## Tray
(dicta-app)») обещает мягкое поведение: «GNOME shows SNI items only with an
AppIndicator extension» — т.е. документировано «иконки нет, приложение живёт», а
код вместо этого убивает старт. Трей — надстройка, а не ядро приложения; отказ
плагина не должен лишать пользователя окна, дашборда и stop-кнопки.
**Fix:** В `main()` обернуть bootstrap в try/catch и продолжать windowed-режим
(лог — только тип, без payload, канон фазы):

```dart
try {
  await TrayController(container, container.read(trayServiceProvider),
      container.read(windowShellProvider)).start();
} catch (error) {
  // Tray is optional: run windowed when the panel/SNI host is unavailable.
  debugPrint('tray bootstrap failed: ${error.runtimeType}');
}
runApp(UncontrolledProviderScope(container: container, child: const DictaApp()));
```

(альтернатива — деградация внутри TrayController: swallowed start, меню-колбэки
не регистрируются).

### WR-03: Живая смена иконки (A1) не проверена против настоящей панели после Task 3

**Severity:** WARNING (validation gap, не доказанный баг)
**File:** `app/lib/core/tray.dart:107-117`; свидетельство — `05-01-SUMMARY.md:173-174`
**Issue:** Smoke Task 2 честно зафиксировал: «Icon colour on recording was NOT
switched in the tracer slice — state→icon wiring arrived in Task 3 (unit-covered)».
Но после Task 3 повторного live-smoke не было: переходы recording/transcribing
проверены только через FakeTrayService, который ничего не знает о том, реально ли
присвоение `trayIcon.icon = ImageAsset.fromAsset(...)` перерисовывает pixmap в
панели KDE (A1 из 05-RESEARCH). План явно требовал живую пробу с fallback-планом:
«A1: если смена trayIcon.icon не обновляет панель — fallback destroy+create внутри
TrayManagerService, шов скрывает; задокументировать в SUMMARY» — fallback не
реализован и не опровергнут, в SUMMARY не задокументирован. Риск: на живой панели
иконка может остаться синей навсегда при том, что юнит-гейт зелёный. UAT-пункт 4
передаёт это человеку, но фазовый гейт «иконка краснеет на записи» не подтверждён
автоматически.
**Fix:** Один живой прогон `flutter run -d linux` при живом демоне: Start из меню
→ глазами/qdbus проверить `IconPixmap` иконки на красные пиксели #E53935, Stop →
синие #35689F. Если присвоение не repaint-ит панель — реализовать
destroy+create-fallback внутри `TrayManagerService.setIconState` (шов скрывает),
задокументировать в SUMMARY.

### WR-04: extractHotkey понимает только двойные кавычки — валидный TOML `hotkey = 'alt+v'` отобразится как «не задан (off)»

**Severity:** WARNING
**File:** `app/lib/core/config_path.dart:30` + `app/lib/features/settings/hotkey_card.dart:112-114`
**Issue:** Regex `^\s*hotkey\s*=\s*"([^"]*)"` матчит только basic-строку. TOML
разрешает literal-строку в одинарных кавычках — демон (настоящий TOML-парсер,
crates/dicta-core) прочитает `hotkey = 'alt+v'` и будет работать по этой
комбинации, а карточка покажет «не задан (off)», т.е. **солжёт о фактическом
состоянии демона** — ровно то, чему HK-01 посвящён («honest card»). Существенно,
что строку `hotkey` пишет НЕ dictad (∉ EDITABLE_KEYS, config.rs:142-160): её
единственный источник — рука пользователя в config.toml (или копия из
config.example.toml с двойными кавычками). Ручная правка с одинарными кавычками —
штатный TOML, не «мусор». План требовал парсить «известный формат» файла, но
формат этого конкретного ключа определяется пользователем, а не set_key.
**Fix:** Расширить regex до обоих TOML-строковых литералов (проверка парности
кавычек не обязательна — формат пишет человек по образцу):

```dart
final RegExp _hotkeyRe = RegExp(r'''^\s*hotkey\s*=\s*["']([^"']*)["']''');
```

+ тест-кейс `extractHotkey("hotkey = 'alt+v'") == 'alt+v'` в config_path_test.dart.

## Info

### IN-01: resolveConfigPath — два edge-расхождения с config.rs

**Severity:** INFO
**File:** `app/lib/core/config_path.dart:19-27`
**Issue:** (а) Rust `std::env::var_os("DICTA_CONFIG")` возвращает `Some("")` для
пустой переменной → демон честно пробует пустой путь; Dart-версия считает пустой
override отсутствующим и уходит в XDG/HOME-путь (config_path.dart:20). (б) Rust
`dirs::config_dir()` требует АБСОЛЮТНЫЙ `XDG_CONFIG_HOME` (relative → None →
`$HOME/.config`); Dart принимает относительный (config_path.dart:23) и прочитает
не тот файл, который читает демон. Оба случая патологические (пустая/относительная
env-переменная), карточка в худшем случае покажет хоткей из чужого файла.
**Fix:** Не срочно. При желании: relative-XDG сверять с Uri и проваливать в HOME
(паритет с dirs), пустой override — оставить как есть (у демона с пустым
DICTA_CONFIG свои проблемы больше, чем у карточки).

### IN-02: Выход из трея при активной записи оставляет демона писать (by design, но нигде не сказано)

**Severity:** INFO
**File:** `app/lib/tray_bootstrap.dart:69-72`
**Issue:** «выход» = tray.dispose + window.destroy; идущая запись НЕ останавливается
и остаётся у демона (по контракту демон владеет жизненным циклом записи; останов —
хоткей демона/dictactl). Это корректное поведение (запись не «висит» — ей владеет
демон), но ни README (EN/RU секции «Tray»), ни пункт меню об этом не предупреждают:
пользователь может считать «выход» = «всё остановлено». Smoke-протокол Task 2
требовал «НИКАКОЙ записи не оставлять» — исполнитель честно останавливал запись
до выхода, поведение по умолчанию не проверялось.
**Fix:** Одна строка в README ×2: «Exit closes the app; an active dictation keeps
running in the daemon — stop it with the daemon hotkey or `dictactl stop`».

### IN-03: Ошибка чтения конфига в HK-карточке не в liveRegion (тишина для AT)

**Severity:** INFO
**File:** `app/lib/features/settings/hotkey_card.dart:131-152`
**Issue:** `_ConfigError` (hkUnavailable + retry) — обычный Text без
`Semantics(liveRegion: true)`. Фаза вводит собственный канон: fetch-ошибки
обязаны быть слышимыми (hf_browser.dart:365-372 — liveRegion добавлен этой же
фазой как закрытие 04-UI-REVIEW п.(д); settings confStatus — liveRegion). Новая
fetch-ошибка карточки — тот же класс события, но AT его не услышит.
**Fix:** Обернуть заголовок ошибки: `Semantics(liveRegion: true, child:
Text(l10n.hkUnavailable, ...))`.

### IN-04: Retry в полёте не виден: hasError-ветка съедает spinner

**Severity:** INFO
**File:** `app/lib/features/settings/hotkey_card.dart:57-61`
**Issue:** Прямое следствие принятого исполнителем Riverpod-отклонения: после
ошибки tap «повторить» → `ref.invalidate` → AsyncLoading с сохранённым прежним
error → `config.hasError` остаётся true → карточка продолжает показывать ошибку
и активную кнопку retry без какого-либо индикатора повторной попытки (в отличие
от data-ветки, где retry честно даёт spinner). Двойной tap плодит
invalidate-циклы (безвредно, но нечестно по UI).
**Fix:** `if (config.hasError && !config.isLoading) _ConfigError(...) else ...`
— ошибка остаётся, но на время retry показывается spinner.

### IN-05: hotkeyConfigTextProvider игнорирует error-поле ответа демона (непаритетно settingsSnapshotProvider)

**Severity:** INFO
**File:** `app/lib/features/settings/hotkey_card.dart:23-25`
**Issue:** `SettingsReply.fromReply(reply).settings` молча отбрасывает
`.error` — тогда как `settingsSnapshotProvider` (providers.dart:57-59)
продвигает его в AsyncError. Практически безвредно: при daemon-side ошибке
snapshot получается all-defaults с `configPath == ''` → карточка честно уходит в
fallback `resolveConfigPath()`; но семантика «ошибка демона тихо превращается в
чтение дефолтного пути» нигде не зафиксирована и отличается от обработки того же
ответа в соседнем провайдере.
**Fix:** Комментарий уже почти есть; добавить явную проверку при желании:
`final parsed = SettingsReply.fromReply(reply); if (parsed.error == null &&
parsed.settings.configPath.isNotEmpty) path = parsed.settings.configPath;`

## Structural Findings (fallow)

Structural-блок в приглашении ревью не передавался — секция опущена.

## Незакрытые вопросы фокуса ревью (сверено, дефектом не признано)

- **Гард иконки/спам/пропуск disconnected:** двойной гард
  (bootstrap `_lastIcon` :34-40 + сервис `_lastIconState` :107-117) согласован,
  disconnected → idle без лишнего вызова — тест `tray_service_test.dart:133-137`
  подтверждает; 10 Гц-спам исключён (poll пишет state каждые 100 мс, но
  setIconState зовётся только на переходах).
- **Riverpod-отклонение HK-карточки:** `config.hasError` + чтение из engineProvider
  напрямую — корректно против гонок (нет вложенного FutureProvider → нет
  disposal-race; `ref.read` в FutureProvider легален, ошибки как данные), тест
  unreadable-path зелёный. Остаточный шов — IN-04.
- **close-to-tray повторный hide/show:** toggle строится на `isVisible()` при
  каждом клике — повторные вызовы идемпотентны; `WindowManagerShell.onCloseRequest`
  защищён от двойной регистрации листенера (window_shell.dart:33).
- **dispose-порядок:** exit = tray.dispose → window.destroy; `_callbacks.clear()`
  во время исполнения exit-колбэка безопасен в Dart; листенеры не снимаются
  явно, но container живёт весь процесс — утечки нет (Timer/StreamController в
  tray-коде == 0, гейт).
- **Фейки:** FakeTrayService/FakeWindowShell достаточны для маршрутных/гардовых
  сценариев; нативные label-порядок и разделители меню покрыты только живым
  DBusMenu-smoke (честно зафиксировано в SUMMARY).
- **errHttp String-placeholder:** generated `errHttp(String code)` — оба call-site
  передают `statusCode.toString()` (hf_browser.dart:95,128) — отклонение
  исполнителя #2 корректно.

---

_Reviewed: 2026-11-22T16:19:18Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
