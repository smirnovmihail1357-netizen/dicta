---
phase: "2"
slug: "dashbord-zapisi-diagnostika-okruzheniya"
# status lifecycle: draft (seeded by plan-phase) → validated (set by validate-phase §6)
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-11-22"
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Source: 02-RESEARCH.md §Validation Architecture (researcher, HIGH confidence, all
> protocol facts verified against dictad main.rs in-session).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `flutter_test` (bundled with Flutter SDK 3.50.1, installed and verified in Phase 1) |
| **Config file** | `app/analysis_options.yaml` (flutter_lints 6, exists from Phase 1) |
| **Quick run command** | `cd app && flutter test` |
| **Full suite command** | `cd app && flutter test && flutter analyze && bash tool/cyrillic_gate.sh` |
| **Estimated runtime** | ~30 seconds (13 existing tests + ~20 new) |

---

## Sampling Rate

- **After every task commit:** `cd app && flutter test`
- **After every plan wave:** `cd app && flutter test && flutter analyze`
- **Per-tick model:** live level comes from the existing 100 ms `DictaStatusNotifier` poll
  (already ≥10 Hz for DASH-01); widget tests compress ticks with
  `tester.pump(const Duration(milliseconds: 100))` (Phase 1 pattern).
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

Task IDs bound to 02-01-PLAN.md (planner, 2026-11-22): rows map to Tasks 1-7 in
order; row 4 carries Threat Ref T-2-01 (inject payload discipline). Legend same as Phase 1.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| Task 1 | 01 | 0 | (infra) | — | N/A | fake factory extraction (8 tests, `implements DictaEngine` == 1) | `cd app && flutter test test/fake_dicta_engine_test.dart test/shell_test.dart test/widget_test.dart` | ✅ | ✅ green |
| Task 2 | 01 | 1 | DASH-01 | — | N/A | tracer: widget (FakeDictaEngine, 3 states) + unit (ring buffer 200/порядок/clear) | `cd app && flutter test test/daemon_engine_framing_test.dart test/dashboard_state_test.dart test/level_history_test.dart` | ✅ | ✅ green |
| Task 3 | 01 | 1 | DASH-02 | — | N/A | widget (button matrix × {idle, recording, transcribing, disconnected}; «распознаю…» spinner) | `cd app && flutter test test/dashboard_buttons_test.dart` | ✅ | ✅ green |
| Task 4 | 01 | 2 | DASH-03 | T-2-01 | inject only sends user's last text via jsonEncode; never logs payload | widget (stop-reply text → card; copy via clipboard seam; inject framing/disabled-при-пустом/error shown) | `cd app && flutter test test/last_text_test.dart` | ✅ | ✅ green |
| Task 5 | 01 | 2 | DASH-04 | — | N/A | widget (drag release → ровно один `set('min_level','0.42')`, формат {:.2}) | `cd app && flutter test test/calibration_test.dart` | ✅ | ✅ green |
| Task 6 | 01 | 3 | DIAG-01 | — | N/A | widget (green/neutral/red/«перепроверить») + smoke по живому демону | `cd app && flutter test test/diagnostics_test.dart && dart run tool/diag_smoke.dart` | ❌ W0 | ⬜ pending |
| Task 7 | 01 | 3 | all | — | N/A | manual smoke | see Manual-Only Verifications | n/a | n/a | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Edge-Probe Resolutions (фаза 2, все 5 граней)

| Requirement | Category (probe) | Resolution |
|-------------|------------------|------------|
| DASH-01 | unclassified | explicit — widget/unit tests above; unknown state string → fallback label |
| DASH-02 | unclassified | explicit — button matrix test; cancel honestly disabled при transcribing (VERIFIED dictad main.rs:940-948: демон поставил бы Idle, но вставка всё равно случится) |
| DASH-03 | unclassified | explicit — framing + widget tests; A1 (inject вставит в активное окно = само окно приложения) покрыт честной подписью + manual UAT |
| DASH-04 | unclassified | explicit — unit+widget; ВАЖНО: `set min_level` разрешён ВО ВРЕМЯ записи (гейт только для ENGINE_KEYS, main.rs:718-754) — UI не должен ложно блокировать калибровку |
| DIAG-01 | unclassified | mixed — automated пункты (сокет/status/модель) explicit-тестами; хоткей+вставка — backstop/manual пункты с подсказками |

---

## Wave 0 Requirements

- [x] `app/test/fake_dicta_engine.dart` — единая фабрика фейков (2 инлайн-копии из
      widget_test.dart/shell_test.dart слиты; `implements DictaEngine` == 1). Фактический
      набор крюков Task 1: мутабельный `statusValue` (скриптование между тиками поллера),
      `setCalls` (записи key/value), `injectTexts` (готов к Task 2), `stopGate`
      (Completer-управляемый stop), `fetchSettingsReply` + `fetchCount`. Скриптуемые
      `stopReply`/`injectReply` добавляются Task 2 вместе с новыми членами интерфейса;
      `buildTestApp` остаётся в `shell_test.dart`.
- [ ] Новые ключи .arb (~25) + `flutter gen-l10n` — до первого виджета (cyrillic gate)
- [ ] Все новые тест-файлы из таблицы выше (стабы)
- [ ] `app/tool/diag_smoke.dart` — по образцу `tool/status_smoke.dart`; живой демон:
      status-тик + безвредный `inject ""` (демон честно ответит error «текст пустой…»,
      вставка не вызывается — main.rs:886-893)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Живой уровень и цвет столбиков на реальном микрофоне | DASH-01 | визуальная полировка, реальный аудиопоток | `cd app && flutter run -d linux`; говорить/молчать — столбик живой |
| Калибровка на живой записи: тишина/речь различимы, порог применяется | DASH-04 | реальная акустика | запись активна → двигать отметку → `set` уходит по отпусканию; перезапуск демона сохраняет min_level в config.toml |
| Inject в реальное окно + кнопка «вставить заново» | DASH-03 | вставка идёт в АКТИВНОЕ окно (при клике по кнопке это само окно приложения) | фокус в стороннее текстовое поле, диктовка по хоткею, затем проверить путь хоткея; кнопка честно подписана |
| Хоткей-путь и вставка в чеклисте диагностики | DIAG-01 | реальный глобальный хоткей | пункты помечены «проверьте руками», подсказки: `dictactl toggle`, вставить текст хоткеем |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending (autonomous mode — orchestrator sign-off; visual items remain user UAT)
