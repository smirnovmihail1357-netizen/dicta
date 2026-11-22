# Phase 1 — Review Disposition

Review: 01-REVIEW.md (commits f00426a..3ed3152, re-checked after fixes)
Fixes: commit 675269c (applied by gsd-code-fixer, verified: flutter analyze clean, 13/13 tests, cyrillic gate 0)

| # | Severity | Finding | Disposition |
|---|----------|---------|-------------|
| W1 | warning | Committed symlink app/lib/l10n/generated → .dart_tool breaks fresh clone/CI | fixed — real generated files committed (l10n.yaml `output-dir: lib/l10n/generated`), imports repointed, `app/tool/cyrillic_gate.sh` added (excludes generated dir), fresh-clone analysis proven |
| W2 | warning | _commandLine catch-all discards the «демон не отвечает… systemctl --user start dictad» hint | fixed — `on SocketException` branch propagates the message into DictaReply.error; catch-all now only handles JSON/EOF |
| W3 | warning | Poll can write `state` after notifier disposal | fixed — `_disposed` flag set in ref.onDispose, guarded after every await |
| I-1 | info | Unclosed StreamController in DaemonEngine | fixed — `dispose()` added, engineProvider registers `ref.onDispose(engine.dispose)` |
| I-2 | info | `1+1==2` placeholder test | fixed — replaced with real DictaApp pump smoke (FakeDictaEngine override, asserts «Дашборд» renders) |
| I-3 (necessity) | info | Dead `base.error` branch — daemon uses `#[serde(flatten)]` | open — informational, branch kept as poison-tolerance; harmless |
| I-4 | info | Fixture contains real user paths | open — accepted per threat T-1-04 (reviewer confirmed: no private text, only paths); noted |
| I-5 | info | `\uXXXX`-escaped Russian strings outside l10n (runtime errors in daemon_engine.dart) | open — intentional: gate counts Cyrillic in lib; runtime exception strings are not UI strings; revisit if gate scope changes |
| I-6 | info | fetchSettings exception inconsistency vs set() | open — cosmetic; both surface errors to UI via different paths; Phase 4 will unify |
| I-7 | info | Fragile `Process.runSync('id', ['-u'])` fallback | open — last-resort path only (XDG_RUNTIME_DIR is always set on this system); noted for Phase 6 Windows port |

**Deferred capability steps (recorded, not silent):** nyquist validate-phase and ui-review deferred — phase 1 is a skeleton with placeholder screens; visual audit is scheduled after Phase 2 (dashboard), nyquist audit after UAT items resolve. security secure-phase runs next (verifier chain).
