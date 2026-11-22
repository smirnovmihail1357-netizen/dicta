# Phase 4 Review Disposition

**Review:** 04-REVIEW.md (3 BLOCKER / 6 WARNING / 6 INFO)
**Fix report:** 04-REVIEW-FIX.md, commits d867622…a0c9750 (9 atomic)
**Battery after fixes:** 138/138 (+7 regression), analyze clean, cyrillic 0, contracts frozen to 9aee17e.

## Disposition

| Finding | Severity | Status | Notes |
|---|---|---|---|
| CR-01 note-as-error kills SetReply success branch | BLOCKER | FIXED d867622 | Note = DATA: success → snapshot + pending-clear + neutral liveRegion; red only on error!=null. Fake now synthesizes daemon-true non-null note; regression test added. Root cause: fake shape diverged from real daemon (main.rs:739) — reviewer caught what the fake-encoding tests couldn't. |
| CR-02 model selection skips refetch | BLOCKER | FIXED d867622 | Success refetches; test proves fetchCount growth + active-marker move. |
| CR-03 unit-suffixed slider values | BLOCKER | FIXED ecee344 | Raw wire values for all 4 ms-keys; units display-only; _sliderValue hardened; drag tests per key. |
| WR-01 pagination offset leak | WARNING | FIXED 8fd9e9d | Reset on find; append on show-more (dedup by repo id). |
| WR-02 one-at-a-time await-gap | WARNING | FIXED 851eea2 | Synchronous slot claim. |
| WR-03 cancel during connect dropped | WARNING | FIXED b659c7a | DownloadCancelGate covers connect window; tested. |
| WR-04 reload timer error-loop | WARNING | FIXED 3b002c8 | Guarded refetches; loop stops on failure. |
| WR-05 filenameOf('..') traversal | WARNING | FIXED 01460fe | Rejected at validation + parse layers; tests. |
| WR-06 «моделей нет» on fetch failure | WARNING | FIXED cc4ef19 | Honest error + retry vs true empty. |
| IN-01 ГБ/МБ literals outside .arb | INFO | WONTFIX | Requires new arb keys + l10n regen + snackbar test rewrite; gate passes as-is. Deferred to a later arb-hygiene pass. |
| IN-02/03/05/06 | INFO | FIXED a0c9750 | Cheap wins batched. |
| IN-04 .part semantics divergence | INFO | WONTFIX | Parity with frozen Rust reference (curl -C -) is the phase contract; changing recovery semantics is out of scope. |

## Accepted deviations (executor)

- FakeDownloadEngine sync RandomAccessFile + renameSync in DownloadManager: fake-async zone compatibility; real seam stays async dart:io; documented in 04-SUMMARY.
- Extra l10n keys (modelDeleteBlocked, confSeconds, confMilliseconds, modelNoteSmall, modelSearchSemantics) — all @-described, gen-l10n ran.
- Summary double-naming (04-01-SUMMARY + 04-SUMMARY) — orchestrator contract + plan output, identical content.

## Deferred

- UI legacy №5 (collapsing long records) → phase 5 (CONTEXT-sanctioned).
- Dashboard busy-gate removal recommendation → intentionally rejected (protects double-stop race; dashboard_buttons_test.dart:83-104 pins it).
- nyquist validate-phase → UAT round (established project convention, phases 2-3).

## Verdict

All blockers and warnings closed; 2 INFO wontfix with justification. Battery 138/138, analyze clean, contracts byte-identical. Ready for verify:post hooks → verifier.
