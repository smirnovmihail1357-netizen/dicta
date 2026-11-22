# Review Disposition — Phase 2

**Review:** 02-REVIEW.md (1 BLOCKER · 4 WARNING · 7 INFO)
**Fix commit:** `b953547` — «app: чиню находки ревью фазы 2 (маркер калибровки, контексты ошибок, таймаут, тест-покрытие)»
**Fixer:** gsd-code-fixer (hy4-flash-free), autonomous directive (user granted full autonomy 2026-11-22).

## Disposition

| ID | Severity | Resolution |
|---|---|---|
| CR-01 | BLOCKER | **FIXED** — маркер калибровки: убрано синхронное чтение `.value` после `ref.refresh` (riverpod 3.5.0 возвращает предыдущий AsyncData); теперь `invalidate` + `await future`, маркер ставится в каноническое значение демона; добавлен тест маркера-после-set через хук `onSet` (ловит именно этот класс бага). |
| WR-01 | WARNING | **FIXED** — контексты ошибок разделены (`DictaCommand` + `lastErrorCommand`): «ошибка вставки» только для inject, нейтральный «ошибка команды» для start/stop/cancel. |
| WR-02 | WARNING | **FIXED** — `if (!mounted) return;` после `await set(...)` до любого обращения к ref; второй гард после рефетча. |
| WR-03 | WARNING | **FIXED** — отдельные ветки `on TimeoutException` в `_commandLine`/`_snapshotCommand`: честное «демон не отвечает (таймаут N с)» вместо «некорректный ответ демона». |
| WR-04 | WARNING | **FIXED** — покрытие: хук `cancelReply` в фабрике; тест cancel с байт-в-байт «запись отменена» (dictad main.rs:947); тест маркера-после-set; тест backoff поллера через хук `statusError`. |
| IN-01 | INFO | **FIXED** — AsyncLoading → нейтральная «данные модели загружаются…», AsyncError → отдельная строка (диагностическая карточка). |
| IN-02 | INFO | **FIXED** — ошибка `SettingsReply.fromReply` повышается до `AsyncError` провайдера, не теряется; broker-контракт задокументирован. |
| IN-04 | INFO | **FIXED** — мёртвый `statusStream` удалён из интерфейса и реализации (grep: ровно 3 пользователя, все — его собственное объявление/подключение). |
| IN-06 | INFO | **FIXED** — единый параметризованный `buildFakeApp` в фабрике вместо шести клонов scaffolds. |
| IN-03 | INFO | **ACCEPTED (necessity)** — четыре недостижимых catch в контроллере: defensive-код против будущих правок seam; тот же класс, что принят в фазе 1 (I-нecessity). |
| IN-05 | INFO | **ACCEPTED (tradeoff)** — русский runtime-текст в `\uXXXX`-эскейпах: плата за кириллический гейт (0 не-комментной кириллицы в lib/); UI-текст при этом весь через .arb. |
| IN-07 | INFO | **ACCEPTED (documented)** — общий `commandInFlight` морозит все кнопки на время команды (до 120 с у stop/inject): defensive-superset матрицы плана; при мёртвом демоне поллер покажет баннер «демон недоступен», заморозка вторична. Осознанно не раздаём per-command флаги — уходит за рамки плана. |

## Проверки после фиксов

- `flutter test` — 48/48 (45 + 3 новых)
- `flutter analyze` — No issues found!
- `cyrillic_gate` — 0
- `git diff --quiet 2714e27 HEAD -- crates/` — пусто; `-- app/pubspec.yaml app/pubspec.lock` — пусто
- Треды T-2-01/02/03/SC перепроверены в ходе ревью — соблюдены.

## Отложенные хуки

- `nyquist validate-phase` — отложен до UAT-прогона (решение фазы 1, сохраняется).
- `ui-review` — ЗАПУСК СЕЙЧАС, после фазы 2, как записано в disposition фазы 1 («ui-аудит после фазы 2»).
