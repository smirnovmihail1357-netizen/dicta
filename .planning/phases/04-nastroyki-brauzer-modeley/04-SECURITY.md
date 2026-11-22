# Phase 4 SECURITY

**Verdict:** SECURED
**Threats Closed:** 7/7
**ASVS Level:** 1 (block_on: high)
**threats_open:** 0

## Threat Verification

| Threat ID | Category | Severity | Status | Evidence (live code) | Test covering it |
|-----------|----------|----------|--------|---------------------|------------------|
| T-4-01 | Tampering/EoP | high | **mitigated** | **Layer 1:** `filenameOf` rejects `.`, `..`, backslash, empty names — `app/lib/core/hf/hf_parse.dart:107-121` (fix 01460fe). **Layer 2:** `DownloadManager.start` rejects `destName` containing `/`, `\`, `..`, `.` before any FS use — `app/lib/core/downloads.dart:81-87`; dest = separator-safe join — `downloads.dart:92-93`. **Layer 3 (UI):** direct link requires http/https AND `filenameOf != null` — `hf_browser.dart:157-161` | `hf_parse_test.dart:89-96`; `download_state_test.dart:279-291`; `models_browser_test.dart:263-294` |
| T-4-02 | Tampering | medium | **mitigated** | `.bin` filter in parseTree — `hf_parse.dart:79`; rename only in `onDone` after full body — `downloads.dart:131` → `_finish` `downloads.dart:160-175`; app never loads/executes model — daemon discovers it itself (`downloads.dart:176-177`) | `hf_parse_test.dart`; `download_state_test.dart:207+` |
| T-4-03 | DoS/quota | low | **mitigated** | One-at-a-time slot claimed **synchronously** before the settings await — `downloads.dart:78,88` (WR-02 fix 851eea2); fetch only on user action (`hf_browser.dart:301-305`, `:335-338`, `:328/:367`); zero Timer/retry loops in hf code | `download_state_test.dart:172-187`, `:229-247` (WR-02 regression); `models_browser_test.dart` |
| T-4-04 | Info Disclosure | medium | **mitigated** | Zero print/debugPrint/log in `app/lib`; error surfaces carry code/type only: `hf_client.dart:33-34`, `downloads.dart:153-155`, `model_store.dart:36`; signed URL held in state but **never rendered** (`hf_browser.dart:439-477`); set values never logged — `daemon_engine.dart:162-186`; `status_smoke.dart:13` prints `error.runtimeType` | final-gate grep `$error` == 0; `models_browser_test.dart:384` (code, not body) |
| T-4-05 | Repudiation | low | **mitigated** | Fresh `HttpClient()` per fetch — `hf_client.dart:29`; per download, redirect chain walked live — `download_engine.dart:105-115`; resolve URL rebuilt per tap — `hf_browser.dart:247`, `:390`, `:432`; no Location storage | URL-parity asserts `models_browser_test.dart` |
| T-4-06 | Tampering/EoP | high | **mitigated** | Triple guard: active model delete disabled — `models_screen.dart:200-201`; destructive confirm — `models_screen.dart:66-84`; `canDelete` re-checked against live snapshot after confirm — `models_screen.dart:88-97`; lexical confinement `File(path).parent.path != destDir` — `model_store.dart:22-27`; errors-as-data — `model_store.dart:31-38` | `model_delete_test.dart:16-34` (active blocked, `../x.bin`, absolute foreign, in-dir allowed); `models_screen_test.dart` |
| T-4-SC | Supply chain | high | **mitigated** | **Verified live:** `git diff 9aee17e HEAD -- crates/ app/pubspec.yaml app/pubspec.lock` → EMPTY; zero curl in app/lib; only pre-existing `Process.runSync('id')` `daemon_engine.dart:21`; zero HttpClient under features/ | Task 9 contract gate; re-verified in this audit |

## Unregistered Flags

None. Review findings WR-05 (traversal — T-4-01, fix 01460fe) and WR-02 (await gap — T-4-03, fix 851eea2) mapped to existing threat IDs and fixed inside the phase with regression tests.

## Residuals (informational)

1. Pre-existing `$error` interpolation at `daemon_engine.dart:61` — SocketException connect-failure OS message only, not a payload leak (connect fails before data is written); unchanged since before phase base; T-2-01/T-3-01 canon intact.
2. Direct link accepts non-`.bin` extensions — faithful parity with frozen reference `main.rs:763-767`; content inert, daemon loads models. One-line follow-up if hardening beyond reference is ever desired.
3. No explicit symlink test in `model_delete_test.dart` — not exploitable (lexical guard; Linux `File.delete()` unlinks the symlink itself); `..` traversal tested.
4. status_smoke debt closed: `tool/status_smoke.dart:13` prints `error.runtimeType` (Task 1, d646e3a).
5. Clipboard untouched this phase (phases 2-3 surfaces only).
6. `Timer.periodic` in settings_screen polls local daemon UDS during reload — allowed by CONTEXT:20, unrelated to T-4-03.

**Phase 4 may ship.**
