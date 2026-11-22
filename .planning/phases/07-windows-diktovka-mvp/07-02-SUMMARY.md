---
phase: 07-windows-diktovka-mvp
plan: "07-02"
subsystem: windows-clipboard-spike
tags: [rust, windows-rs, clipboard, delayed-render, wm-renderformat, ci, spike, win-04]
requires:
  - phase: 07-01
    provides: "windows-crate 0.58 pin, CI Windows rust job, dictad.exe artifact"
provides:
  - "spike/delayed_render — standalone Win32 experiment (9-measurement stdout protocol) + README, compiled green on windows-latest"
  - "CI compile gate for the spike in .github/workflows/windows-build.yml (rust job)"
  - "07-SPIKE-RESULT.md scaffold (docs, uncommitted on main — for orchestrator) with pinned 'факт: ' row labels"
affects: [07-02 continuation (Tasks 2-4), 07-VALIDATION UAT]
tech-stack:
  added: [windows 0.58 (spike-local lock), Win32 delayed-render APIs, SendInput with dwExtraInfo]
  patterns: ["promise clipboard + WM_RENDERFORMAT (no blind timer)", "standalone [workspace] crate outside root workspace", "stdout-measurement protocol for human-run GUI spike"]
key-files:
  created:
    - spike/delayed_render/Cargo.toml
    - spike/delayed_render/Cargo.lock
    - spike/delayed_render/src/main.rs
    - spike/delayed_render/README.md
    - .planning/phases/07-windows-diktovka-mvp/07-SPIKE-RESULT.md (scaffold, docs → main)
  modified:
    - .github/workflows/windows-build.yml
key-decisions:
  - "Spike is a standalone crate with empty [workspace] + own Cargo.lock to stay out of the root workspace; windows 0.58 pinned to match 07-01."
  - "Added Win32_System_Memory, Win32_Graphics_Gdi and Win32_UI_Input_KeyboardAndMouse features beyond the plan's four — required for GlobalAlloc, WNDCLASSW/RegisterClassW and SendInput (Rule 2 correctness)."
  - "windows-rs maps SetClipboardData(NULL) to Err (NULL considered invalid) — protocol honestly treats Err as Win32-success and verifies via IsClipboardFormatAvailable + GetClipboardOwner, never GetLastError."
  - "Type of owner window, settle timing and Chromium re-render count are NOT guessed: Task 3 is deliberately not implemented until real measurements exist (Must-NOT 8)."
requirements-completed: []
actuals:
  tokens: 8493
  tasks: 1
  commits: 1
  plan_head_before: 63130140897eb8214560ee1df825a21ef28517ee
  plan_head_after: da0842ebf03dbf50c17c1ecd8519e94373902bcd
metrics:
  duration: 13m
  completed: 2026-11-22
  status: halted
status: halted
branch: windows
---

# Phase 07 Plan 02: Windows диктовка MVP Summary

**Готов спик-инструментарий delayed render (Win32 promise + WM_RENDERFORMAT) с протоколом 9 живых замеров и зелёным CI compile-гейтом; production-вставка намеренно НЕ написана — задача остановлена на предусловии Must-NOT 8.**

## Результат: EXECUTION INCOMPLETE (штатный останов, НЕ провал)

Выполнена **1 из 4 задач**. Task 1 завершена полностью; Task 2 — чекпоинт физического
прогона на Windows-машине пользователя (агент не имеет GUI-машины, не подделывал
результаты); Task 3 остановлена шагом 0 предусловия; Task 4 не начиналась.

## Accomplishments

1. **Task 1 (Wave 1) — сделано.** `spike/delayed_render`: standalone bin;
   `Cargo.toml` + свой `Cargo.lock` (windows 0.58); `src/main.rs` (~350 строк)
   с baseline-замером, promise `SetClipboardData(CF_UNICODETEXT, NULL)`,
   четырьмя оптаут-форматами, обработкой `WM_RENDERFORMAT` / `WM_RENDERALLFORMATS`,
   режимами окна `message-only` / `hidden` и вставки `user` / `send` (SendInput +
   `dwExtraInfo`), 5-секундным логом settle и самотестом `WM_RENDERALLFORMATS`.
   Текст диктовки в логи не попадает никогда (только длины/id/времена).
2. **README-протокол** (`spike/delayed_render/README.md`): сборка, 2×2×5 прогонов,
   9 замеров, PowerShell `Get-Clipboard`, Win+V-проверка, честная шапка «не production».
3. **`07-SPIKE-RESULT.md` scaffold** (docs, не закоммичен на windows): 9 строк с
   закреплённым лейблом `факт: <заполнить>` (17 вхождений `факт`), колонки
   «ожидание»/«факт», секция «Выводы для inject_win», гейт перехода к Task 3.
4. **CI compile-гейт спика** в `.github/workflows/windows-build.yml` (rust-джоба,
   шаг после тестов). Живой прогон: **run 45068498544 → success**, шаг
   `Spike delayed render — compile gate` зелёный с первого раза; обе джобы зелёные.
   Локально: `cargo check --target x86_64-pc-windows-gnu --manifest-path ...`
   проходит без ошибок; `cargo test --workspace` — зелёная.

## Task Commits

1. **Task 1: spike + protocol + CI gate** — `da0842e` (feat, ветка windows).

План-метadata: нет отдельного docs-коммита (docs или не коммитились на windows
по хореографии, или остались за оркестратором на main). Run-id:
`/tmp/opencode/rust-run-ids.txt` → `07-02-spike:45068498544`.

## Files Created/Modified

- `spike/delayed_render/Cargo.toml` — standalone-манифест, `[workspace]`,
  windows 0.58 + 7 фич, без synth/IPC/audio-крейтов.
- `spike/delayed_render/Cargo.lock` — свой lock (windows 0.58.0).
- `spike/delayed_render/src/main.rs` — эксперимент, stdout-замеры.
- `spike/delayed_render/README.md` — протокол прогона на Windows-машине.
- `.github/workflows/windows-build.yml` — шаг compile-гейта спика.
- `.planning/phases/07-windows-diktovka-mvp/07-SPIKE-RESULT.md` — scaffold
  (untracked в windows-worktree; Отдать docs-коммиту на main).

## Decisions Made

- Спик — чистый Win32 без enigo; standalone-крейт вне workspace с собственным lock.
- Feature-расширение (Memory/Gdi/Input) — техническая необходимость API windows 0.58.
- Никаких догадок о типе окна/settle/рендерах: данные только от живого прогона.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Добавлены feature `Win32_System_Memory`,
`Win32_Graphics_Gdi`, `Win32_UI_Input_KeyboardAndMouse` сверх четырёх из плана**
- **Found during:** Task 1
- **Issue:** Без Memory нет `GlobalAlloc` для реального хендла при рендере,
  без Gdi нет `WNDCLASSW`/`RegisterClassW`, без Input нет `SendInput` (режим send).
- **Fix:** Добавлены в `spike/delayed_render/Cargo.toml`; закоммичено.
- **Verification:** `cargo check --target x86_64-pc-windows-gnu` + зелёный CI-гейт.

**2. [Rule 3 - Blocking] Пустой `[workspace]` в манифесте спика**
- **Found during:** Task 1 (генерация Cargo.lock)
- **Issue:** Cargo автоматически втягивал спик в родительский workspace корня.
- **Fix:** Добавлен `[workspace]` — спик стал независимым, свой lock сгенерирован.
- **Verification:** `cargo generate-lockfile` прошёл; корневой workspace не изменён.

**3. [Rule 1 - Bug] `hInstance: hinstance.into()` и печать `GetLastError.0`**
- **Found during:** Task 1 (кросс-проверка на windows-gnu target)
- **Issue:** `GetModuleHandleW` возвращает HMODULE, поле WNDCLASSW ждёт HINSTANCE;
  `WIN32_ERROR` не реализует Debug.
- **Fix:** Явное `.into()` и печать числового `.0`; убрана interpolation `e.code()`.
- **Verification:** `cargo check --target x86_64-pc-windows-gnu` — 0 errors.

**Total deviations:** 3 auto-fixed. Impact: только список фич спика иtypes;
production-код, протокол, hive Linux и transport не тронуты.

## Issues Encountered

- CI-раннер headless: GUI-замеры физически невоспроизводимы (A1) — добавлен
  compile-гейт, живые данные ожидаются с машины пользователя.
- В `Cargo.toml` спика слово «enigo» в комментарии ломало отрицательный гейт
  Task 1 (`! grep -rq enigo`) — переформулировано, гейт зелёный.

## Next Phase Readiness / Что осталось (EXECUTION INCOMPLETE)

**Точка останова: Task 3, шаг 0.** `07-SPIKE-RESULT.md` содержит маркеры
`<заполнить>` → production-вставка не написана (Must-NOT 8 соблюдён).

### Точная инструкция пользователю

1. На Windows-машине: `cd spike\delayed_render` → `cargo build --release`
   (или собрать/взять код с ветки `windows`; артефакт `dictad.exe` НЕ нужен).
2. Прогнать: `.\target\release\dicta-spike-delayed-render.exe --mode=user --window=message-only`,
   затем `--window=hidden`, затем `--mode=send` — по всем 5 целевым окнам
   (Notepad / Chromium / Windows Terminal / elevated-Notepad / зажатый модификатор).
3. Скопировать строки stdout `SPIKE | ...` построчно в колонку «факт» всех 9 строк
   `07-SPIKE-RESULT.md`, заполнить «Выводы для inject_win», убрать все `<заполнить>`.
   Дополнительно: PowerShell `Get-Clipboard` + Win+V-проверка (замер 9).
4. docs-коммит `docs(07-02): spike result — delayed render measurements` на main.

### После прогона остаётся

- **Task 3** — тело `inject_win` (enigo + delayed render по замерам + 4 оптаута +
  честный UIPI-отказ + release-паттерн + восстановление baseline) и focus-развилка;
  CI run-id ключ `07-02-inject`.
- **Task 4** — README-заметка ×2, полная батарея, аудит против 23049d6,
  финальный CI (`07-02-final`), SUMMARY на main.
- **UAT** (07-VALIDATION:59-65, human_needed): 4 кейса вставки, живая диктовка,
  подключение Flutter-приложения, Win+V/облако, CPU-латентность.

---
*Phase: 07-windows-diktovka-mvp*
*Status: HALTED — ожидает физического прогона спика (Must-NOT 8)*
