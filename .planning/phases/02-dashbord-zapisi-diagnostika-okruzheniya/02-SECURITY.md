---
phase: 02-dashbord-zapisi-diagnostika-okruzheniya
date: 2026-11-22
asvs_level: 1
threats_total: 4
threats_mitigated: 4
threats_accepted: 0
threats_open: 0
---

# Phase 02 Security Audit — Дашборд записи + диагностика окружения

Scope: `app/` only (commits 603d410..b953547, база 2714e27, включая ревью-фиксы b953547).
Rust/daemon и протокол заморожены: `git diff --quiet 2714e27 HEAD -- crates/` пуст — вне скоупа.
ASVS L1 verification (mitigation present in cited file), с L2 boundary-проверками там,
где mitigation мог быть обойдён параллельным кодовым путём (call-site аудит clipboard-шва,
grep всей поверхности логирования, count фабрик фейков).

## Mitigation Table

| Threat | Severity | Disposition | Status | Evidence |
|--------|----------|-------------|--------|----------|
| T-2-01 payload утечка через логи/печать (Info disclosure) | high | mitigate | CLOSED | Сериализация — только `jsonEncode`: `buildInjectLine` (`app/lib/core/engine.dart:99-100`), единственный путь inject-строки в провод (`daemon_engine.dart:145`). Поверхность логирования пуста: grep `print(\|debugPrint\|stdout\|logger` по `lib/ tool/ test/` → 4 хита, ни один не payload — uid-резолв (`daemon_engine.dart:20`), `status_smoke.dart:10` (имя state), `diag_smoke.dart:12-13` (state name + connected flag), `diag_smoke.dart:26` (фиксированная строка); комментарий-замок «never log reply payloads» (`daemon_engine.dart:58`), error-пути несут тип ошибки и путь сокета. Файловых записей в features/tool нет (grep `File(\|writeAs` — 0 релевантных). `diag_smoke.dart:16` шлёт ровно `inject(text: '')` — диктованный текст не генерируется и не печатается. Framing-регрессия `daemon_engine_framing_test.dart:63-79` — кавычки/бэкслеши/переводы строк round-trip byte-exact + inject с app; виджет-тест `last_text_test.dart:83,112,141` — кнопка шлёт ровно lastText, при пустом состоянии `injectTexts` пуст |
| T-2-02 тамплинг буфера обмена (Tampering) | medium | mitigate | CLOSED | Копирование — только по явному клику: шов `clipboardWriterProvider` = `Clipboard.setData` (`dicta_control.dart:147-148`); L2 call-site аудит: единственный production-вызов — `last_text_card.dart:55`, внутри `onPressed:` с гардом `hasText && !busy` (null при недоступности). Автоматических/фоновых вызовов нет (`grep clipboardWriterProvider lib/ test/` → определение, единственный call-site, test-override в `fake_dicta_engine.dart:140`). Чтения буфера нет вовсе: `grep Clipboard lib/` → единственный хит `setData` (`dicta_control.dart:148`). Clipboard-restore выполняет демон своими настройками; UI лишь честно подписывает (dashInjectHint, `last_text_card.dart:79`) |
| T-2-03 inject сверхдлинного текста >5000 (Tampering) | low | mitigate | CLOSED | Клиент НЕ перевалидирует длину (валидация принадлежит демону — lock): grep `5000\|length` по inject-пути `lib/features/ lib/core/` → только ring-buffer/гистограмма, проверок длины нет. Заведомо невалидное не шлётся: кнопка недоступна при null/пустом тексте (`last_text_card.dart:68`, `hasText` = `text != null && text.isNotEmpty`), в полёте — тоже. Отказ демона показывается дословно как данные (`last_text_card.dart:90-94` — «Daemon error text is data: shown verbatim, never invented»), UI не выдумывает успех и не глотает ошибку; empty-inject отвергается демоном до вставки — smoke подтверждает живьём |
| T-2-SC supply chain pub.dev (Tampering) | high | mitigate | CLOSED | `git diff --quiet 2714e27 HEAD -- app/pubspec.yaml app/pubspec.lock` — IDENTICAL (пустой diff, байт-в-байт); `git diff --quiet 2714e27 HEAD -- crates/` — тоже пуст (протокол заморожен). Гистограмма/калибровка — стандартные виджеты: `calibration_card.dart` = LayoutBuilder + CustomPainter, grep chart-пакетов → 0. Единая фабрика фейков: `grep "implements DictaEngine" test/*.dart \| wc -l` == 1 (обе инлайн-копии фазы 1 слиты в `fake_dicta_engine.dart`) |

## Residual Risks

- **(informational) diag_smoke stderr-печать объекта ошибки:** `diag_smoke.dart:28` —
  `stderr.writeln('DIAG SMOKE FAIL $error')`. Сегодня исключения движка обёрнуты и несут
  только тип transport-ошибки и путь сокета (`daemon_engine.dart:57-61`) — payload недостижим.
  Но это единственное место, где нефиксированная строка уходит в вывод: если будущая
  error-ветка когда-нибудь вложит reply-payload в исключение, T-2-01 получит утечку через
  этот готовый канал. Рекомендация фазе 3+: печатать `error.runtimeType`/фиксированную строку,
  не объект целиком.
- **(informational) lastText живёт в памяти и UI до конца сессии:** распознанный текст
  хранится в `DictaControlState` и рендерится в `SelectableText` — это и есть фича (DASH-03),
  не лог; in-memory, не персистится (LevelHistory и control — оба in-memory, буфер/текст
  переживают только жизнь виджета). Граница проведена корректно: status.text НИКОГДА не
  рендерится как «последний текст» (grep `status.text` по `lib/features/` → 0 хитов) — единственный
  источник lastText это ответы stop/inject (`dicta_control.dart:85,127`).
- **(унаследовано из фазы 1, не меняется)** T-1-03 DICTA_SOCK env-trust: same-user сессия,
  принято в фазе 1; фаза 2 не добавила новых env-поверхностей.

## Verdict

Все 4 заявленных треда верифицированы в реализованном коде (4 mitigated, 0 accepted, 0 open).
SUMMARY.md фазы не содержит секции `## Threat Flags`; незарегистрированных новых
поверхностей атаки в скоупе не обнаружено (все новые файлы — UI-слой поверх существующего
same-user сокета; новых команд демону, новых зависимостей, новых env/файловых путей — ноль).
Ревью-фиксы b953547 (lastErrorCommand, таймаут, маркер калибровки) скоуп аудита не расширили.
`threats_open: 0`.

**Verdict: SECURED.** Аудитор: gsd-security-auditor (фон, автономный режим), 2026-11-22.
