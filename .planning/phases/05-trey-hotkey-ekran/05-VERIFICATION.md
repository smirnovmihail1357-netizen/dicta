---
phase: 05-trey-hotkey-ekran
verified: 2026-11-22T18:30:00Z
status: human_needed
score: 12/12 must-haves verified
covered_files:
  - .planning/phases/05-trey-hotkey-ekran/05-01-PLAN.md
  - .planning/phases/05-trey-hotkey-ekran/05-01-SUMMARY.md
  - README.md
  - README.ru.md
  - app/assets/tray/idle.png
  - app/assets/tray/recording.png
  - app/lib/core/config_path.dart
  - app/lib/core/daemon_engine.dart
  - app/lib/core/providers.dart
  - app/lib/core/tray.dart
  - app/lib/core/window_shell.dart
  - app/lib/features/history/history_screen.dart
  - app/lib/features/models/hf_browser.dart
  - app/lib/features/models/model_store.dart
  - app/lib/features/models/models_screen.dart
  - app/lib/features/settings/hotkey_card.dart
  - app/lib/features/settings/settings_screen.dart
  - app/lib/features/shell/shell.dart
  - app/lib/l10n/app_ru.arb
  - app/lib/l10n/generated/app_localizations.dart
  - app/lib/l10n/generated/app_localizations_ru.dart
  - app/lib/main.dart
  - app/lib/tray_bootstrap.dart
  - app/pubspec.lock
  - app/pubspec.yaml
  - app/test/config_path_test.dart
  - app/test/fake_dicta_engine.dart
  - app/test/fake_tray_service.dart
  - app/test/fake_window_shell.dart
  - app/test/fixtures/config_example_toml.txt
  - app/test/fixtures/config_live_no_hotkey.txt
  - app/test/history_screen_test.dart
  - app/test/hotkey_card_test.dart
  - app/test/models_screen_test.dart
  - app/test/shell_test.dart
  - app/test/tray_service_test.dart
  - app/test/window_shell_test.dart
  - tools/gen_tray_icons.dart
covered_digest: "v4:sha256:81230c455dce182ba08251edaeba07a394c9bb7cb5bd1ec7f656f4a44c99da7b"
behavior_unverified: 0
overrides_applied: 0
human_verification:
  - test: "Живой трей глазами на KDE: запустить app при живом демоне — иконка dicta в панели сразу после старта; физический клик по иконке (левая/правая) — зафиксировать фактическое поведение панели (ожидание: панель сама открывает меню)"
    expected: "иконка видна; клик любой кнопкой открывает экспортированное DBusMenu; пункты показать/скрыть-старт-стоп-отмена-настройки-выход работают против живого демона"
    why_human: "физический клик и глаза пользователя не инжектируются в Wayland-сессии верификатора; исполнитель прогнал DBusMenu-контракт и живой DBus-замер IconPixmap (WR-03), но подпись поведения панели — только человеком"
  - test: "Подпись принятого SNI-отклонения TRAY-01 (оркестратор 2026-11-22): «левый клик = показать/скрыть окно» реализован пунктом меню «показать/скрыть окно», т.к. tray_manager на Linux не сообщает клики по иконке"
    expected: "пользователь подтверждает, что доступ к окну по клику через пункт меню его устраивает; отклонение подписано в UAT"
    why_human: "это продуктовое решение, требующее живого клика и согласия пользователя — не проверяется прогоном"
  - test: "Close-to-tray живого окна: нажать крестик — окно скрылось, процесс жив, иконка осталась; затем пункт «выход» — процесс завершился полностью (ps пуст)"
    expected: "крестик = скрыть (не убить); «выход» = tray dispose + window destroy, процесс мёртв; запись демона не прерывается выходом (by design, README)"
    why_human: "unit-тесты доказали контракт шва (close→hide, destroy только из меню), но поведение живого X11/Wayland-окна глазами не проверяется grep'ом"
  - test: "HK-карточка на живом конфиге: вкладка «Настройки» → группа «система»"
    expected: "карточка показывает текущую evdev-комбинацию или штатное «не задан (off)» (в живом конфиге строки hotkey нет), DE-таблицу 4 строки, предупреждение «применится после рестарта демона»"
    why_human: "живой config.toml пользователя и рендер карточки — живой рантайм; фикстурные тесты доказали оба сценария на verbatim-текстах"
  - test: "Демон-хоткей при живом приложении: нажать Alt+V (или живую комбинацию) при запущенном dicta-app"
    expected: "одно срабатывание — запись стартует/стопается один раз, без двойного stop (гонка HK-02 исключена: приложение хоткей не регистрирует, гейт hotkey_manager==0 живо измерен)"
    why_human: "живая запись через демон + физическая клавиша; гонка доказана отсутствием второй регистрации, но живое срабатывание — человеку"
  - test: "Сверка 8 must-NOT из CONTEXT.md (без глобального хоткея в app, без записи config.toml, Rust байт-в-байт, без второго полл-цикла, без клонов фабрик фейков, plugin-каналы вне тестов, без toml-пакета, без новых rail-пунктов)"
    expected: "все 8 подтверждены живыми гейтами верификации (см. таблицу прогонов) и подписываются пользователем одним чек-листом"
    why_human: "гейты уже зелёные прогонами; подпись — формальность UAT-раунда (канон фаз 3-4)"
---

# Phase 5: Трей + хоткей-экран — Verification Report

**Phase Goal:** Приложение живёт в фоне: запись и окно доступны из трея, хоткей-экран честно показывает, как устроен глобальный хоткей (ROADMAP.md Phase 5)
**Verified:** 2026-11-22T18:30:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

**Верифицировано живыми прогонами верификатора, заявления SUMMARY не принимались на веру.**
Диапазон: b81178b..HEAD (исполнение 74dcce2..cd81c58, review-фиксы 331f79f..23dafbd, UI-фиксы 8dfde38..f780071 — все коммиты присутствуют в git log).

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence (живой прогон) |
|---|-------|--------|--------------------------|
| 1 | Трей едет на существующих тиках dictaStatusProvider/dictaControlProvider; нового полл-цикла нет | ✓ VERIFIED | `grep Timer\|Stopwatch\|Socket.connect lib/core/tray.dart lib/tray_bootstrap.dart` → 0; `_container.listen(dictaStatusProvider)` (tray_bootstrap.dart:88); disconnected → idle (:90) |
| 2 | Крестик = hide (setPreventClose + onWindowClose→hide); destroy ТОЛЬКО из пункта меню | ✓ VERIFIED | call-site grep: `window.destroy()` в lib ровно 1 — tray_bootstrap.dart:122 (путь «выход»); поведение: `window_shell_test.dart` «close request hides without destroying (TRAY-02)» и «destroy happens only through an explicit exit call», `tray_service_test.dart` «TRAY-02: the X hides…» + «honest exit: tray disposed first, then window destroyed» — все в зелёной батарее 178/178 |
| 3 | SNI-отклонение TRAY-01: «левый клик» реализован пунктом меню «показать/скрыть окно» (панель сама открывает меню) — принято оркестратором 2026-11-22, UAT-подпись обязательна | ✓ VERIFIED (реализация+документация) | пункт меню есть (tray.dart, лейбл l10n.trayShowHide); отклонение зафиксировано в PLAN objective, 05-VALIDATION.md:14, SUMMARY:151-157, README.md:236-241, README.ru.md; WR-03 живой DBus-замер IconPixmap в tray.dart:107-118 (208 синих/0 красных → 0/208 → 208/0). Физический клик — UAT-подпись (см. human_verification) |
| 4 | Меню зовёт ТОЛЬКО существующие start/stop/cancel через DictaControlNotifier; протокол/демон/Rust байт-в-байт | ✓ VERIFIED | `engineProvider` в tray_bootstrap → 0; `dictaControlProvider.notifier` (:141-153) + WR-01 гард `_menuCommandInFlight` (тест «WR-01: a rapid double click issues only one command» зелёный); `git diff --quiet b81178b HEAD -- crates/ adapters/ systemd/` → RUST-FROZEN |
| 5 | Иконки 2 шт (idle #35689F, recording #E53935); recording И transcribing → красная; disconnected → синяя; setIconState только на переходах | ✓ VERIFIED | gen_tray_icons.dart:79-82 (RGBA 53,104,159 / 229,57,53); `iconFor` switch (tray.dart:28); гард `if (state == _lastIconState) return` (tray.dart:119); тест «icon follows status transitions only (guard, red on transcribing)» зелёный; `dart tools/gen_tray_icons.dart && git diff --quiet -- app/assets/tray/` → ICONS-DETERMINISTIC |
| 6 | HK-карточка: конфиг ТОЛЬКО читается; «не задан (off)» — штатно; hotkey ∉ set-команды | ✓ VERIFIED | `writeAs\|openWrite\|FileMode` в config_path+hotkey_card → 0; `set('hotkey')` в lib → 0; extractHotkey null → hkNotSet; config_path_test + hotkey_card_test (фикстуры config_example_toml.txt / config_live_no_hotkey.txt) зелёные |
| 7 | Приложение НЕ регистрирует глобальный хоткей (HK-02) | ✓ VERIFIED | `grep -rn hotkey_manager pubspec.yaml pubspec.lock lib/` → 0 |
| 8 | Plugin-каналы только из prod-main и core/tray.dart + core/window_shell.dart; фейки их не дёргают | ✓ VERIFIED | tray_manager в lib → 1 (только core/tray.dart), в test/ → 0; buildFakeApp `TrayService? tray, WindowShell? window` null-means-no-override (fake_dicta_engine.dart:189-190,206-207); main.dart → `bootstrapTray(container)` до runApp (:27-32); WR-02 fallback в windowed-режим (тест «WR-02: a failing tray bootstrap never propagates» зелёный) |
| 9 | Payload-privacy: `$error` → error.runtimeType; ноль print/логов пейлоадов | ✓ VERIFIED | `$error` в daemon_engine.dart → 0; `print(` в lib → 0; bootstrap-лог — только error.runtimeType |
| 10 | Вкладка из трея через selectedTabProvider, НЕ _selected; rail не тронут | ✓ VERIFIED | `_selected` в shell.dart → 0; SelectedTabNotifier → 2 (класс+провайдер, канон history.dart); NavigationRailDestination → 4; `selectedTabProvider.notifier.select(3)` (tray_bootstrap.dart:118); тест «settings menu item shows the window and selects the Settings tab» зелёный |
| 11 | UI-наследие фазы 4 закрыто | ✓ VERIFIED | `HTTP ${error.statusCode}` → 0; `delete failed` в model_store → 0; `} GB`/`} MB` → 0; modelSearchTitle → 1; histExpand → 2 (SelectableText сохранён → 2, HIST-03); liveRegion в hf_browser → 3; excludeSemantics → 1; MergeSemantics в settings → 4 (не просел); тесты history/models расширены и зелёные |
| 12 | Батарея зелёная, analyze «No issues found!», cyrillic 0, implements 1/1/1 | ✓ VERIFIED | `flutter test` → **178/178 All tests passed** (живой прогон верификатора); `flutter analyze` → No issues found!; `bash tool/cyrillic_gate.sh` → 0; implements DictaEngine/TrayService/WindowShell: lib 1/1/1, test 1/1/1 |

**Score:** 12/12 truths verified (0 present-behavior-unverified — каждая поведенческая истина покрыта именным зелёным тестом; живые бэкстопы — в human_verification ниже)

### Required Artifacts

| Artifact | Status | Evidence |
|----------|--------|----------|
| app/lib/core/tray.dart (шов + TrayManagerService + iconFor) | ✓ VERIFIED | единственный tray_manager-файл в lib; interface plugin-free; WR-03 live proof :107-118 |
| app/lib/core/window_shell.dart (шов + WindowManagerShell) | ✓ VERIFIED | implements == 1 в lib; window_manager-legacy поверх main.dart:7 precedent |
| app/lib/tray_bootstrap.dart (TrayController + bootstrapTray) | ✓ VERIFIED | WR-01 гард, WR-02 try/catch, destroy единственный call-site |
| app/lib/core/config_path.dart | ✓ VERIFIED | паритет config.rs:173-178 вкл. IN-01 (абсолютный XDG, пустой DICTA_CONFIG=unset); WR-04 одинарные кавычки; ноль toml-пакета |
| app/lib/features/settings/hotkey_card.dart | ✓ VERIFIED | врезка `const HotkeyCard()` в settings_screen.dart:617 (группа «система», без rail-пункта) |
| app/assets/tray/{idle,recording}.png + tools/gen_tray_icons.dart | ✓ VERIFIED | ICONS-DETERMINISTIC (перегенерация байт-в-байт, живой прогон) |
| app/lib/features/shell/shell.dart (selectedTabProvider) | ✓ VERIFIED | см. truth 10 |
| 24 .arb-ключа + generated | ✓ VERIFIED | якорный grep → 24 ровно |
| фейки tray/window | ✓ VERIFIED | implements 1/1, null-means-no-override |
| тесты tray_service/window_shell/config_path/hotkey_card + фикстуры | ✓ VERIFIED | все в батарее 178/178 |
| README.md + README.ru.md секции Tray | ✓ VERIFIED | «## Tray (dicta-app)» == 1; «## Трей (dicta-app)» == 1; SNI-поведение, close-to-tray, честный выход, GNOME-оговорка, HK-указатель — все в тексте |

### Key Link Verification

| From | To | Via | Status |
|------|----|----|--------|
| app/lib/main.dart | app/lib/tray_bootstrap.dart | `bootstrapTray(container)` до `runApp(UncontrolledProviderScope)` | ✓ WIRED (main.dart:27-32) |
| tray_bootstrap.dart | core/providers.dart | `container.read(trayServiceProvider/windowShellProvider)` | ✓ WIRED (:42-46) |
| tray_bootstrap.dart | dicta_control.dart | `dictaControlProvider.notifier` + WR-01 гард | ✓ WIRED (:141-153) |
| tray_bootstrap.dart | shell.dart | `selectedTabProvider.notifier.select(3)` + window.show() | ✓ WIRED (:118) |
| window_shell.dart | window_manager legacy | setPreventClose + onWindowClose→hide; destroy только из меню | ✓ WIRED |
| hotkey_card.dart | config_path.dart | configPath из ответа демона первично → fallback resolveConfigPath(); extractHotkey | ✓ WIRED |
| settings_screen.dart | hotkey_card.dart | `const HotkeyCard()` в _group(l10n.confGroupSystem) | ✓ WIRED (:617) |
| tools/gen_tray_icons.dart | app/assets/tray/*.png | детерминированная генерация | ✓ WIRED (git diff пуст после перегенерации) |

### Behavioral Spot-Checks (единственный полный прогон батареи)

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Полная батарея | `cd app && flutter test` | **+178: All tests passed!** | ✓ PASS |
| Анализатор | `cd app && flutter analyze` | No issues found! | ✓ PASS |
| Кириллица-гейт | `bash tool/cyrillic_gate.sh` | 0 | ✓ PASS |
| Живой демон-смоук | `cd app && dart run tool/status_smoke.dart` | SMOKE OK state=idle, exit 0 | ✓ PASS |
| Контракт заморозки | `git diff --quiet b81178b HEAD -- crates/ adapters/ systemd/` | пусто (RUST-FROZEN) | ✓ PASS |
| Детерминизм иконок | `dart tools/gen_tray_icons.dart && git diff --quiet -- app/assets/tray/` | ICONS-DETERMINISTIC | ✓ PASS |
| Единственная новая зависимость | `git diff b81178b HEAD -- app/pubspec.yaml app/pubspec.lock` | pubspec: +tray_manager 0.9.0, +assets/tray/; lock: ровно 1 блок tray_manager (8 строк) | ✓ PASS |

Именные поведенческие тесты (все в зелёном прогоне 178/178): close→hide без destroy; honest exit (dispose→destroy); show/hide toggle оба направления; меню→FakeDictaEngine через DictaControlNotifier; WR-01 двойной клик = одна команда; WR-02 краш-безопасный bootstrap; иконка по переходам с гардом, transcribing→red; вкладка «настройки» из трея; config-парсинг (example/no-hotkey/off/одинарные кавычки/edge-пути); HK-карточка (off/значение/недоступный путь).

### Requirements Coverage

| Requirement | Описание | Status | Evidence |
|-------------|-----------|--------|----------|
| TRAY-01 | SNI-трей: меню, левый клик, иконка состояния | ✓ SATISFIED (с принятым SNI-отклонением — подпись в UAT) | truths 1,3,5 + live smoke WR-03 |
| TRAY-02 | close-to-tray, честный выход | ✓ SATISFIED | truth 2 |
| HK-01 | Экран хоткея: комбинация, путь, предупреждение | ✓ SATISFIED | truth 6 |
| HK-02 | Приложение не регистрирует глобальный хоткей | ✓ SATISFIED (живая гонка — UAT п.8) | truth 7 |

Орифаненных требований нет: REQUIREMENTS.md мапит на фазу 5 ровно TRAY-01/02, HK-01/02 — все заявлены планом и закрыты.

### Anti-Patterns Found

| File | Pattern | Severity |
|------|---------|----------|
| — | TBD/FIXME/XXX/TODO/HACK/placeholder в файлах фазы | ℹ️ ни одного (grep по всем 24 файлам фазы — пусто) |
| app/lib/core/config_path.dart:50 | `return null` | ℹ️ не стаб — штатный «нет hotkey-строки» по контракту extractHotkey (тестом покрыт) |

Долговых маркеров нет; блокеров нет.

### Advisory (Info)

1. **SUMMARY:135,207 говорит «170 tests»** — цифра на момент Task 6 до review-фиксов; живая батарея сегодня **178/178** (review-серия добавила 8 тестов, 05-REVIEW-DISPOSITION сходится). Не расхождение цели, просто устаревшее число в Performance-секции.
2. **PLAN Task 6 ссылается на «05-VALIDATION.md:58-64»** для UAT-листа из 8 пунктов; файл VALIDATION — 49 строк, его Manual UAT (:31) сжат до 4 пунктов. Полный 8-пунктовый лист живёт в SUMMARY:253-262 — требование задачи выполнено, ссылка в плане дрейфовала.

### Human Verification Required

Backstop-истины (канон VALIDATION: explicit уже доказан юнитами/гейтами выше; backstop — живой рантайм глазами). Полный список с инструкциями — **05-UAT.md** (8 пунктов, статус testing):

1. **Живой трей на KDE глазами** — иконка после старта, физический клик (зафиксировать поведение панели), start/stop живьём, смена иконки на записи.
2. **Подпись SNI-отклонения** (TRAY-01 буква → пункт меню; оркестратор 2026-11-22).
3. **Close-to-tray живого окна + честный выход** (крестик → скрыто/жив; «выход» → процесс мёртв).
4. **HK-карточка на живом конфиге** («не задан (off)» — штатно).
5. **Демон-хоткей при живом app: одно срабатывание без двойного stop** (HK-02 backstop).
6. **Сверка/подпись 8 must-NOT из CONTEXT.md** (все уже автодоказаны гейтами — подпись формальность).

### Gaps Summary

Автоматических гэпов нет: 12/12 истин подтверждены живыми прогонами, все контракты целы
(RUST-FROZEN, единственная зависимость tray_manager 0.9.0, implements 1/1/1, hotkey_manager 0,
иконки детерминированы, plugin-изоляция, конфиг read-only, ноль второго полл-цикла, residual
runtimeType погашен). Фаза честно уходит в **human_needed**: живой SNI-трей, физический клик
по иконке, живое close-to-tray и демон-хоткей при живом app — бэкстопы, которые нельзя
прогнать без глаз и рук пользователя; они уже сформулированы в 05-UAT.md вместе с
обязательной подписью принятого SNI-отклонения.

---

_Verified: 2026-11-22T18:30:00Z_
_Verifier: the agent (gsd-verifier)_
