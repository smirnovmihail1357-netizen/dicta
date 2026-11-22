---
status: testing
phase: 05-trey-hotkey-ekran
source: [05-VERIFICATION.md, 05-VALIDATION.md, 05-01-SUMMARY.md]
started: 2026-11-22T18:30:00
updated: 2026-11-22T18:30:00
---

# Phase 5 UAT — Трей + хоткей-экран

Automated evidence (12/12 must-haves) is in 05-VERIFICATION.md: батарея **178/178**
(перепущена верификатором живьём), analyze «No issues found!», cyrillic gate 0,
контракт заморозки `git diff b81178b HEAD -- crates/ adapters/ systemd/` пуст,
pubspec/lock-diff = только tray_manager 0.9.0, implements DictaEngine/TrayService/WindowShell
== 1/1/1 (lib и test), hotkey_manager 0 (HK-02), иконки детерминированы (перегенерация
байт-в-байт), plugin-изоляция (tray_manager в test/ == 0), конфиг read-only, ноль второго
полл-цикла, `$error` residual погашен, security SECURED 7/7 (05-SECURITY.md), review-фиксы
WR-01..04 + IN-01..03 закрыты тестами, UI-аудит 18/24 без блокеров (05-UI-REVIEW.md).

**Запуск приложения:** `cd app && flutter run -d linux` при живом демоне
(`systemctl --user start dictad`, `dart run tool/status_smoke.dart` → SMOKE OK state=idle).
Каждый пункт отмечай `result:` и подписью.

## Tests

### 1. Иконка dicta видна в трее KDE сразу после старта приложения

expected: синяя иконка (idle #35689F) появляется в системном лотке сразу после запуска
app при живом демоне; tooltip «Dicta».
result: [pending]
source: VERIFICATION human_verification 1; SUMMARY UAT 1
auto_evidence: "SNI-регистрация и IconPixmap 22×22 проверены живым DBus-замером исполнителя (WR-03, tray.dart:107-118) — глазами подтверждается видимость и tooltip"

### 2. Клик по иконке открывает меню + ПОДПИСЬ SNI-отклонения

expected: клик ЛЮБОЙ кнопкой — панель KDE сама открывает меню (6 пунктов: показать/скрыть
окно, начать запись, остановить, отмена, настройки, выход). Зафиксируй фактическое
поведение левой/правой кнопки твоей панели.
**Здесь же подписывается принятое отклонение от буквы TRAY-01** (оркестратор,
2026-11-22): tray_manager на Linux не сообщает клики по иконке, поэтому «левый клик =
доступ к окну» реализован пунктом меню «показать/скрыть окно». Если поведение панели и
компромисс тебя устраивают — подпиши.
result: [pending]
source: VERIFICATION human_verification 1-2; VALIDATION Manual UAT; SUMMARY UAT 2
auto_evidence: "DBusMenu-контракт (тот же путь, что использует панель) прогнан end-to-end живым смоуком; физический клик в Wayland-сессии агента не инжектируется"

### 3. Меню: «показать/скрыть окно» работает в обе стороны; start/stop/cancel доходят до демона

expected: показ/скрытие окна в обе стороны; «начать запись» — демон state=recording
(иконка краснеет), «остановить»/«отмена» — state=idle; НИКАКОЙ записи не остаётся висеть
после проверки.
result: [pending]
source: SUMMARY UAT 3
auto_evidence: "маршрутизация меню → DictaControlNotifier → движок покрыта юнитами (в т.ч. WR-01 двойной клик = одна команда); живой смоук исполнителя прогонял start→recording, stop→idle по DBusMenu"

### 4. Иконка краснеет при записи/распознавании и синеет в idle (без 10 Гц-спама)

expected: переходы recording/transcribing → красная #E53935, idle/disconnected → синяя;
переключение только на переходах состояния.
result: [pending]
source: SUMMARY UAT 4
auto_evidence: "гард переходов и transcribing→red покрыты тестом «icon follows status transitions only»; живой замер IconPixmap (208/0 → 0/208 → 208/0) в WR-03; глазами — итоговое поведение панели"

### 5. Крестик окна: окно скрылось, приложение живо в трее; «выход» полностью завершает процесс

expected: крестик = скрыть (процесс жив, иконка осталась); пункт «выход» — процесс мёртв
(`ps aux | grep dicta_app` пуст), SNI разрегистрирован. Выход НЕ прерывает активную запись
демона — by design (README.md:228-230).
result: [pending]
source: VERIFICATION human_verification 3; SUMMARY UAT 5
auto_evidence: "контракт шва доказан юнитами (close→hide без destroy; destroy только из пункта «выход», единственный call-site tray_bootstrap.dart:122); живое окно — глазами"

### 6. HK-карточка на живом конфиге

expected: «Настройки» → группа «система»: текущая evdev-комбинация или штатное
«не задан (off)» (в живом конфиге строки hotkey нет — это норма), DE-таблица 4 строки
(KWin/Hyprland/Sway/GNOME), предупреждение «применится после рестарта демона»,
подсказка про input-группу. Конфиг только читается (гейт writeAs==0 живой).
result: [pending]
source: VERIFICATION human_verification 4; SUMMARY UAT 6
auto_evidence: "оба сценария (off / значение) покрыты фикстурными тестами на verbatim-текстах; read-only доказан гейтом"

### 7. Сверка 8 must-NOT из CONTEXT.md

expected: все 8 подтверждены (уже автодоказаны живыми гейтами верификатора —
проверь глазами и подпиши одним списком):
1. Без глобального хоткея в приложении (hotkey_manager 0 в pubspec/lock/lib) ✓ гейт
2. Без записи config.toml (writeAs/openWrite/FileMode = 0) ✓ гейт
3. Без правок протокола/демона/Rust (crates/adapters/systemd байт-в-байт к b81178b) ✓ гейт
4. dicta-settings не тронут (входит в RUST-FROZEN) ✓ гейт
5. Без print/логов пейлоадов в lib (print = 0, `$error` residual погашен) ✓ гейт
6. Кириллица в lib вне generated — только .arb (cyrillic gate 0) ✓ гейт
7. Без новых пакетов кроме tray_manager 0.9.0, без toml-пакета (lock-diff = 1 блок) ✓ гейт
8. Без новых rail-пунктов, rail = 4 пункта (NavigationRailDestination = 4) ✓ гейт
result: [pending]
source: VERIFICATION human_verification 6; SUMMARY UAT 7
auto_evidence: "все 8 закрыты живыми гейтами в 05-VERIFICATION.md; подпись — формальность раунда"

### 8. Демон-хоткей при живом приложении: одно срабатывание, без двойного stop

expected: при запущенном dicta-app нажать живую комбинацию демона (Alt+V / твою) —
запись стартует/останавливается ОДИН раз; гонки двойного stop нет. Это backstop HK-02:
приложение хоткей не регистрирует (гейт), но живую однократность подтверждает человек.
result: [pending]
source: VERIFICATION human_verification 5; SUMMARY UAT 8
auto_evidence: "отсутствие второй регистрации доказано гейтом hotkey_manager==0; живое срабатывание — только руками"

## Notes

- Гашение после каждого пункта: ни висящей записи, ни оставленного процесса.
- Пункты 2 и 8 — обязательные подписи (SNI-отклонение + HK-02 backstop); остальные —
  подтверждение уже автодоказанных контрактов на живом рантайме.
