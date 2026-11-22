---
phase: 6
phase_name: windows-ci-karkas
audit_date: 2026-11-22
security_asvs_level: 1
block_on: high
threats_total: 5
threats_open: 0
verdict: SECURED
phase_base: d07763a
audited_range: d07763a..windows
---

# Phase 6 Security Audit — Windows CI + каркас

Вердикт: **SECURED** — 5/5 угроз закрыты, подтверждено живым кодом на ветке `windows`
(ASVS L1 + L2-проверка границ форка). Аудитируемый дифф: `git diff d07763a..windows`
(10 коммитов e217f4a..23049d6, включая 4 review-фикса WR-01..04 до 872fbbf + docs).
SUMMARY-заявления проверены живыми grep-пробами, не приняты на веру.

## Threat Register — верификация

| ID | Категория | Severity | Диспозиция | Статус | Evidence |
|----|-----------|----------|------------|--------|----------|
| T-6-01 | Spoofing | high | mitigate | CLOSED | Пайп-имя только в константе kDictaPipe (windows_engine.dart:20); resolvePipePath читает ТОЛЬКО DICTA_PIPE-override (:25-29), DICTA_SOCK в app/lib присутствует только в замороженном daemon_engine.dart:12,17 (Linux-ветка, недостижима на Windows — форк engineProvider providers.dart:18-19 `Platform.isWindows ? WindowsPipeEngine() : DaemonEngine(...)`); pipe-литерал в app/lib == 1; Platform.is в app/lib == 2 (providers.dart:18 + config_path.dart:29), features/ — ноль; DACL-контракт сервера записан в README.md/README.ru.md (якорь «named pipe DACL»/«DACL именованного канала» = 1/1) |
| T-6-02 | Tampering (supply chain) | high | mitigate | CLOSED | Точный пин `dart_ipc: 2.0.2` БЕЗ caret (pubspec.yaml:37); pubspec.lock закоммичен с sha256 (pubspec.lock:76-82); живой lock-diff содержит РОВНО три добавленных заголовка — dart_ipc 2.0.2 / win32 6.4.0 / ffi_leak_tracker 0.1.2 (все с sha256), версионного дрейфа существующих пакетов нет (полный дифф lock просмотрен); legitimacy-аудит RESEARCH принят планом |
| T-6-03 | Information Disclosure | medium | mitigate | CLOSED | windows-build.yml не экспортирует env и не содержит secret-упоминаний (grep `$env:`/`printenv`/`secret` — ноль); артефакт собирается ТОЛЬКО из build/windows/x64/runner/Release/* (windows-build.yml:32), retention 14 дней, timeout-minutes: 30 (WR-04, commit 872fbbf); payload-privacy: error-строки транспорта = тип+путь+runtimeType (windows_engine.dart:88-95), константы _badReply/_timeoutHint (:33-41), SetReply-ветки не логируют key/value (:203-221), диктовка (inject text) не попадает ни в одну error-строку |
| T-6-04 | Tampering/Injection | medium | mitigate | CLOSED | Пути только из Platform.environment с injectable-параметрами: windowsConfigPath({String? appData}) (config_path.dart:47-48), empty/null → honest-фолбэк `dicta\config.toml` (тесты windows_paths_test.dart:18,23); LOCALAPPDATA — read-only резолв (hf_catalog.dart:63-65); Process/shell в новом коде == 0 (единственный Process.runSync — замороженный Linux `id -u` daemon_engine.dart:21, байт-в-байт к d07763a); backslash-паритет WR-03 покрыт тестом (windows_paths_test.dart:73) |
| T-6-SC | Tampering (freeze) | high | mitigate | CLOSED | Живые git-диффы: crates/ — ПУСТ; daemon_engine.dart + ci.yml + release.yml — 0 байт к d07763a; lock.diff НЕПУСТ и замыкание = dart_ipc→win32→ffi_leak_tracker (ffi/meta существующие не тронуты); adjacency — workflow-коммит e217f4a первый в d07763a..windows; пин без caret + sha256 в lock исключают тихую смену версии |

## Побочные проверки скоупа аудита

- **destroy-единственность (WR-02):** поздний коннект-успех реапится единожды —
  `connectFuture.then((socket) => socket.destroy()).ignore()` (windows_engine.dart:73-78);
  штатный сокет уничтожается ровно одним `destroy()` в finally (:106-108); тест
  _CountingSocket.destroyCount == 1 живой (windows_engine_test.dart:29-34,138-147).
- **Payload-privacy греп-поверхность:** все 9 `error:`-присваиваний в windows_engine —
  константы / _transportMessage / _timeoutHint; jsonEncode только в исходящих командных
  строках; пейлоад в ошибки не отражается.

## Threat Flags (SUMMARY)

Секции `## Threat Flags` в 06-01-SUMMARY.md нет — новая поверхность исполнителем
не задекларирована. Post-plan коммиты (WR-01..04) сверены с 06-REVIEW-DISPOSITION.md:
все FIXED живым кодом, поверхность атаки не расширяют.
Unregistered flags: none.

## Фазовые гейты (замеры аудита, все зелёные)

- git diff d07763a windows -- crates/ → пусто; -- daemon_engine.dart ci.yml release.yml → 0 байт
- lock-diff: +dart_ipc/+win32/+ffi_leak_tracker, вне замыкания == 0, дрейф версий == 0
- Platform.is в app/lib == 2; implements DictaEngine: lib == 2 / test == 1
- workflow: env/secrets == 0, артефакт только Release/*, timeout-minutes: 30
- README-якоря: «named pipe DACL» == 1 (README.md), «DACL именованного канала» == 1 (README.ru.md)

## Residuals (informational)

1. **T-6-01 residual (дизайн, фаза 7):** до появления DACL-сервера фазы 7 любой
   локальный процесс может preempt-создать `\\.\pipe\dicta` раньше сервера —
   клиент каркаса подключится к спуфу. Контракт (security descriptor только
   текущий user SID, сервер через interprocess) записан в README ×2 и в
   windows_engine.dart:14-17; реализация — обязательство фазы 7.
2. **DICTA_PIPE env-seam в проде (windows_engine.dart:26-27):** процесс, способный
   выставить окружение приложения, перенаправит путь пайпа. Паритет с Linux
   DICTA_SOCK, тот же уровень доверия; заявлено планом как test-only seam;
   тесты инжектируют через конструктор, не через env.
3. **_transportMessage фолбэк `error.toString()` (windows_engine.dart:56):** для
   чужих IOException теоретически может вывести текст ОС-ошибки (не пейлоад
   диктовки — payload privacy не нарушен).
4. **IN-01 (WONTFIX, фаза 7):** set/fetchSettings Windows-ветки не достигают живого
   пайпа (сервера ещё нет) — паритет-долг фазы 7, шов и фейки готовы.

threats_open: 0
