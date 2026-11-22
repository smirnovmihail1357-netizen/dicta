---
phase: 03-istoriya-diktovok
date: 2026-11-22
asvs_level: 1
threats_total: 4
threats_mitigated: 4
threats_accepted: 0
threats_open: 0
---

# Phase 03 Security Audit — История диктовок

Scope: `app/` only (commits 305ed18..HEAD: de9e94c, 37c1f3b, 7a4fb90, 9ce9071,
5d1b47b, 3cb280e, 5eb2d90; база фазы = phase-base.txt = 305ed18c, ревью-фиксы 3cb280e
включены). Rust/daemon и протокол заморожены: `git diff --quiet 305ed18 HEAD --
crates/` пуст — вне скоупа. ASVS L1 (mitigation present в цитированном файле) с
L2 boundary-проверками там, где mitigation мог быть обойдён параллельным путём
(call-site аудит clipboard-шва по обоим production-точкам, grep всей поверхности
логирования lib+tool+test, контрактный diff манифеста к базе фазы).

## Mitigation Table

| Threat | Severity | Disposition | Status | Evidence |
|--------|----------|-------------|--------|----------|
| T-3-01 payload утечка через логи/печать (Info disclosure) | medium | mitigate | CLOSED | Поверхность логирования grep `print\|debugPrint\|stdout\|stderr\|logger\|log(` по `lib tool` → 4 файла, ни один не несёт payload: uid-резолв `daemon_engine.dart:20` (унаслед., фаза 1), `status_smoke.dart:10-12` (имя state; error-объект — унаслед. residual, transport-ошибки несут только тип + путь сокета), `diag_smoke.dart:12,21,26,29` — фиксированные строки state/connected, **и только `error.runtimeType`** в catch (`:29`, комментарий «Type only: never print the error object (02-SECURITY residual)») — residual 02-SECURITY ПОГАШЕН в этой фазе. Новый код фазы чист: grep по `lib/features/history`, `lib/core/theme_ext.dart`, `dicta_control.dart` → 0 хитов. Захват — единственная ссылка `historyProvider` в `dicta_control.dart:106`, внутри `stop()` под гардом `error == null && text != null && trim().isNotEmpty`; catch-ветка не журналирует. `status.text` не читается (grep по lib → 0). Снеки/комментарии payload не несут |
| T-3-02 тамплинг буфера обмена (Info disclosure) | medium | mitigate | CLOSED | Шов `clipboardWriterProvider` = `Clipboard.setData` (`dicta_control.dart:165-166`). L2 call-site аудит: ровно 2 production-вызова, ОБА внутри `onPressed:` явной кнопки — `last_text_card.dart:56` (унаслед.) и новый `history_screen.dart:112` (кнопка `histCopyButton-${id}`, комментарий «Explicit click only.», `onPressed` всегда активна — копия по клику без гейтов, по плану). Фоновых/автоматических вызовов нет; в тестах шов перехватывается `onCopy`-override (`fake_dicta_engine.dart:144`). Чтения буфера нет: `grep "Clipboard\." lib test tool` → единственный хит `setData` |
| T-3-03 данные истории не покидают процесс (Info disclosure) | medium | mitigate | CLOSED | Grep `SharedPreferences\|sqflite\|writeAsString\|File(\|HttpClient\|dio\|http.\|path_provider` по `lib` → 0 релевантных хитов (хиты `jsonEncode` — только провод same-user сокета, `core/engine.dart:94-100` + `daemon_engine.dart:97,139,157`, унаслед. фаза 1). Store in-memory: doc-замок `history.dart:1-5` («In-memory… Deliberately not persisted… HIST-04»), `build()` = const пустой список, никакой дисковой/сетевой поверхности. Удаление — локальный `remove(entry.id)` (`history_screen.dart:161-167`, комментарий «Local removal only: no daemon command»); re-insert — только через `dictaControlProvider.inject()` под `busy`-гардом `commandInFlight` (`history_screen.dart:130-138`). Переживает только жизнь процесса — HIST-04 |
| T-3-SC supply chain (Tampering) | high | mitigate | CLOSED | `git diff --quiet 305ed18 HEAD -- crates/ app/pubspec.yaml app/pubspec.lock` → IDENTICAL (exit 0, пустой diff, байт-в-байт к базе фазы). Полный список изменённых путей `305ed18..HEAD` (минус .planning) = 21 файл, все — lib/test/tool/README; pubspec/manifest/systemd/adapters отсутствуют в диффе. Нулевые новые зависимости: intl 0.20.3 уже стоял (прямой dep с фазы 1), экран/store/ThemeExtension — только stdlib Flutter/Riverpod/intl. `implements DictaEngine` в test/ == 1 (фабрика fake_dicta_engine.dart расширена, не клонирована — ревью 03-REVIEW.md гейт ✓). Installs в фазе нет |

## Residual Risks

- **(informational, унаследован) `status_smoke.dart:12` печатает error-объект целиком**
  (`stderr.writeln('SMOKE FAIL $error')`). Сегодня transport-исключения несут только тип
  ошибки и путь сокета (`daemon_engine.dart:57-61`) — payload недостижим. Рекомендация
  фазе 4+: перевести на `error.runtimeType`. Не блокирует.
- **(informational) журнал + lastText живут в памяти и UI до конца сессии** — это сама фича
  (HIST-01..04), не лог: SelectableText рендерит запись пользователю, ничего не персистится
  (T-3-03 grep = 0). Граница та же, что принята в 02-SECURITY.md.
- **(informational) head-derived id переиспользуется после удалений** (IN-04, disposition:
  FIXED doc+test) — не security-поверхность; отмечено, чтобы фаза 4+ не ключевалась на id
  сквозь очистки.

## Verdict

Все 4 заявленных треда верифицированы живыми командами в реализованном коде
(4 mitigated, 0 accepted, 0 open; blocking: 0). Незарегистрированных новых поверхностей
атаки не обнаружено: новых зависимостей — ноль (манифест байт-в-байт), новых команд
демону — ноль (re-insert через существующий `inject()`, удаление локально), новых
env/файловых/сетевых путей — ноль, глобальных хоткеев — ноль (CONTEXT №4). Ревью-фиксы
3cb280e (MergeSemantics, echo-тест, doc-комментарии) скоуп аудита не расширили.
Унаследованный residual 02-SECURITY (diag_smoke печать error-объекта) в этой фазе закрыт.
Контрактная база: `git diff 305ed18 HEAD -- crates/ pubspec.yaml pubspec.lock` пуст.

`threats_open: 0`

**Verdict: SECURED.** Аудитор: gsd-security-auditor (фон, автономный режим), 2026-11-22.
