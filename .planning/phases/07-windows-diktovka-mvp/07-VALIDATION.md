---
phase: 7
phase_name: Windows диктовка MVP
requirements: [WIN-03, WIN-04]
status: draft
validated_by: plan-checker
---

# Validation Strategy — Phase 7

## Requirements & Verification Approach

| Req | Verification Strategy | Edge-probe resolution |
|---|---|---|
| WIN-03 | Mixed: код — explicit (unit-тесты парсинга hotkey-спеки и структуры capture чистыми функциями; CI-компиляция `cargo build --no-default-features -p dictad` на windows-latest; Linux-батарея не деградирует — git diff main по cfg-гейтам), живая диктовка — backstop (UAT на Windows- машине пользователя: микрофон → whisper → текст) | **mixed**: explicit для кода + backstop `statement`: «пользователь продиктовал на живой Windows фразой и получил текст с той же моделью/конфигом», verification: backstop |
| WIN-04 | Mixed: код — explicit (SDDL-строитель unit-тест: SY + user-SID, не WD; enigo-комбо порядок Press/Click/Release мок-трейтом; протокол не изменён — существующие framing/golden тесты), delayed render + 4 кейса — backstop (спик-протокол §5 RESEARCH на Windows-машине; 4 кейса: обычное/elevated-UIPI/зажатый модификатор/Chromium — физически manual) | **mixed**: explicit для логики + backstop `statement`: «спик 07-SPIKE-RESULT.md выполнен на живой машине, 4 кейса вставки подписаны», verification: backstop |

## Mandatory Gates (фаза-уровень)

1. `cargo test --workspace` (Linux) зелёный после каждого Rust-таска — Linux-батарея не деградирует.
2. `git diff main -- crates/` показывает ТОЛЬКО cfg-развилки + новые _win-модули; Linux-поведение байт-в-байт (ci.yml не тронут).
3. CI: расширенный windows-build.yml — Flutter-джоба + Rust-джоба (`dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`, `cargo build/test --no-default-features -p dicta-core -p dictad`) зелёный на ветке windows.
4. `flutter test` (Linux) — батарея не падает (фаза не ломает Flutter; IN-02-остаток: hotkeyConfigTextProvider error-ветка + тесты).
5. Спик: `spike/delayed_render` + `07-SPIKE-RESULT.md` существуют ДО production inject_win (Must-NOT 8; план обязан ставить inject-задачу с precondition на спик-результат).
6. Windows-порты путей: %APPDATA% конфиг / %LOCALAPPDATA% модели (dirs::data_local_dir развилка — A-находка RESEARCH §10).
7. DACL: SDDL `D:P(A;;GA;;;SY)(A;;GA;;;<owner-SID>)` — unit-тест строки; пайп создаётся сервером (закрывает preempt-спуф residual 06-SECURITY).
8. Логи: текст диктовки НЕ логируется на Windows-пути (Must-NOT 5 строже существующего — RESEARCH Security Domain).

## Per-Task Verification Map

Task IDs: TBD — планёр привязывает при создании 07-01-PLAN.md / 07-02-PLAN.md (ожидаемая раскладка ниже).

| Behavior | Test command | Task ID |
|---|---|---|
| Hotkey-парсинг «alt+v» → (MOD_ALT\|MOD_NOREPEAT, VK_V) | `cargo test -p dicta-core hotkey_win` | TBD |
| Capture-структура 16кГц mono / level>0 (без реального звука) | `cargo test -p dicta-core capture_win` | TBD |
| Windows-сборка CPU-only | CI windows-latest Rust-джоба | TBD |
| SDDL DACL-строка | `cargo test -p dictad transport_win` | TBD |
| Enigo-комбо порядок | `cargo test -p dicta-core inject_win` | TBD |
| Протокол не изменён | `flutter test` (framing/golden фазы 1-6) | TBD |
| Linux не деградировал | `cargo test --workspace` + `git diff main -- crates/` | TBD |
| Спик-артефакты | `test -f spike/delayed_render/Cargo.toml && test -f .../07-SPIKE-RESULT.md` (после прогона) | TBD |
| IN-02-остаток Flutter | `flutter test` ≥ 194 | TBD |

## Nyquist Sampling

- Каждая задача: automated `<verify>` с `<fails_when>` (гейты красные на HEAD до работы — актуальные [HEAD=… EXPECT=…] замеряются планёром живьём).
- Волны: per-task commit (Rust: cargo test --workspace на Linux), per-wave: push → CI windows; phase gate: UAT-протокол.
- Backstop-истины (живой звук, спик-замеры, 4 кейса вставки) — НЕ помечать passed автоматически: verifier честно ставит human_needed, UAT-файл поглощает (06-UAT U-3 подключение к живому серверу — поглощается фазой 7, RESEARCH A8).

## Wave 0 Checklist (пред-условия первого кодового таска)

- [ ] phase-base.txt снят с research-коммита фазы 7 (a434ff6) — база заморозки
- [ ] interprocess пин: 2.4.8 (locked 4) или 2.4.9 с обоснованием A6 — решение планёра
- [ ] CPU-only: whisper-rs без default features на Windows (Vulkan сломан upstream #3750 — RESEARCH §7)
- [ ] cfg-гейтинг существующих модулей (capture/hotkey/inject/atspi + dictad main transport + dictactl UDS) — список RESEARCH §3
- [ ] CI: Rust-джоба в windows-build.yml (scoped -p, НЕ --workspace — dicta-ui не собирается)

## UAT (поглощается 07-UAT.md при verify)

1. Живая диктовка на Windows: хоткей без окна → фраза → текст в буфине/поле (WIN-03 backstop).
2. Спик-протокол §5: 9 замеров delayed render → 07-SPIKE-RESULT.md.
3. 4 кейса вставки: обычное окно / elevated (UIPI) / зажатый модификатор / Chromium.
4. Подключение Flutter-приложения к живому Windows-демону (поглощение 06-UAT U-3, A8).
5. Win+V/облако оптаут-форматы + восстановление исходного буфера.
