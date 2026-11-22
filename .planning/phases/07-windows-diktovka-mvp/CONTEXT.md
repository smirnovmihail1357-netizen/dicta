# Phase 7 Context — Windows диктовка MVP (WIN-03, WIN-04)

Дата: 2026-11-22 · Оркестратор: GLM-5.3 (автономная директива пользователя, вопросы не задавать) · Исполнение: hy4-flash-free субагенты.

## Locked (решения, не подлежащие пересмотру планёром)

1. **Ветка**: вся работа фазы 7 — на ветке `windows` (база 23049d6); кодовые коммиты → windows, .planning-доки → main. Merge в main НЕ делаем.
2. **Протокол заморожен**: тот же JSON-протокол демона поверх named pipe `\\.\pipe\dicta` (interprocess 2.4.8); команды status/settings/set/start/stop/toggle/cancel/inject/panel без изменений; Flutter-сторона (WindowsPipeEngine фазы 6) НЕ переписывается, только подключается к живому серверу.
3. **Linux-путь нетронут**: `cfg(windows)`/`cfg(unix)` развилки в Rust; поведение Linux байт-в-байт сохраняется (гейты сравнения с main).
4. **Стек (из .planning/research/STACK.md, подтверждено research-флагом ROADMAP)**: cpal 0.18.2 (WASAPI), enigo 0.6.1 (SendInput KEYEVENTF_UNICODE), interprocess 2.4.8 (пайп-сервер + DACL), whisper — существующий движок dicta-core (та же модель/конфиг; whisper.cpp уже собирается на Windows CI). Новые зависимости Rust — только эти три + их замыкание; UI-зависимости Flutter НЕ растут.
5. **Спик ОБЯЗАТЕЛЕН до фиксации реализации вставки** (research-флаг ROADMAP): delayed render `SetClipboardData(NULL)` + `WM_RENDERFORMAT` — экспериментальная проверка settle-таймингов и форматов ДО написания production-кода вставки (PITFALLS.md: settle ~400 мс, Handy #502 класс).
6. **DACL-контракт из фазы 6** (README-якорь «named pipe DACL»): security descriptor пайпа — только текущий пользователь (owner SID), SYSTEM; закрывает residual «preempt-спуф пайпа» из 06-SECURITY.
7. **Пути**: конфиг `%APPDATA%\dicta\config.toml` (паритет resolveConfigPath фазы 5), модели `%LOCALAPPDATA%` (обязательство data_local_dir из README фазы 6); env-шов `DICTA_PIPE` (паритет DICTA_SOCK, kDictaPipe дефолт).
8. **Хоткей в ДЕМОНЕ** (паритет Linux-решения): RegisterHotKey + скрытое message-only окно («без открытия окна» = без видимого UI); приложение НЕ регистрирует свои глобальные хоткеи.
9. **Кириллица**: Flutter-код — только через .arb (гейт cyrillic_gate как раньше); Rust-код — ASCII-идентификаторы, русские строки допустимы как в существующих crates (конвенция dictad).
10. **UI-hint**: новых Flutter-виджетов почти нет (подключение к живому серверу); ui-plan-gate → block:false ожидаем; ui-аудит фазы 7 — решает оркестратор после исполнения (фаза 6 SKIPPED с обоснованием).

## Verified facts (живой код, проверить до планирования)

- Демон живой (Linux): idle, /run/user/1000/dicta.sock; dictad структура: Handler + Arc<Mutex<Engine>> + run_stop-воркер (fin: compare_exchange Transcribing→Idle); ENGINE_KEYS гейт reload при не-Idle (dictad main.rs:718-754); EDITABLE_KEYS config.rs:142-160; ENGINE_KEYS config.rs:163-170.
- Протокол: ответ `{"base":{state,level,seconds,text,error},"panel":bool,"settings":...}`; `set`-ответ ВСЕГДА несёт свежий снимок + note (main.rs:737-741).
- Формат ответа status: text:None всегда (main.rs:828-833) — история/текст только через stop/inject.
- Linux-транспорт демона: UDS через std::os::unix (заменяется cfg(unix)); Windows-транспорт — interprocess named pipe listener (сервер), паритет семантики «соединение на команду» НЕ обязателен: держать соединения несложно, но проще сохранить «одна команда — одно соединение» для паритета поведения.
- Захват/тишина/autostop/whisper: dicta-core audio/capture/filter/stt — платформенно-независимы, кроме устройств ввода (cpal уже используется? ПРОВЕРИТЬ: какой бэкенд сейчас — если cpal, Win-ветка почти бесплатна; если hwloc/alsa-sys напрямую — больше работы).
- ВСТАВКА Linux: wl-paste/wl-copy + wtype/atspi (inject.rs) — всё cfg(unix); Windows-ветка вставки новая целиком (enigo + clipboard delayed render).
- Flutter батарея 194/194 (32 файла), analyze 0, cyrillic 0; ветка windows = 23049d6, CI зелёный (windows-build.yml, flutter 3.50.1, артефакт zip).
- Паритет-долги фазы 6 (IN-01/IN-02): set/fetchSettings полный на Windows (автоматично при живом демоне), error-поле settingsSnapshotProvider (малый Flutter-долг — можно закрыть в этой фазе одним коммитом).

## Must-NOT (запреты)

1. НЕ трогать Linux-пути без cfg-развилки; НЕ менять протокол; НЕ менять Flutter-транспорт (windows_engine.dart фазы 6).
2. НЕ регистрировать глобальные хоткеи в Flutter-приложении.
3. НЕ портировать dicta-ui (layer-shell overlay) — Windows-оверлей вне вехи; вставка без видимого окна.
4. НЕ вводить новые Flutter-зависимости; Rust — только cpal/enigo/interprocess (+замыкание, все уже в STACK-исследовании).
5. НЕ хранить аудио/тексты диктовок на диске; payload не логируется (паритет T-контрактов).
6. НЕ делать merge windows→main; НЕ пушить без CI-проверки (push ок — CI зелёный).
7. НЕ нарушать DACL-контракт: пайп доступен только текущему пользователю.
8. НЕ пропускать спик: реализация вставки пишется ТОЛЬКО после спик-результата.

## Inputs (входы планирования)

- ROADMAP Phase 7 SC (4 пункта); REQUIREMENTS WIN-03 (:56), WIN-04 (:57).
- .planning/research/STACK.md: cpal 0.18.2 MSRV 1.85 (ПРОВЕРИТЬ наш rustc), enigo 0.6.1, interprocess 2.4.8; CI-рецепт.
- .planning/research/PITFALLS.md: delayed render settle ~400мс; UIPI (SendInput не проходит в elevated — fallback сценарий); зажатые модификаторы (release-паттерн); Chromium (KEYEVENTF_UNICODE ок, но paste-эвент требует settle).
- README-фазы 6: DACL-контракт + data_local_dir обещание.
- 06-SECURITY residuals: preempt-спуф (закрывается DACL), DICTA_PIPE env-seam.
- Живой app/: windows_engine.dart (kDictaPipe, таймауты 2/30/120), config_path.dart (resolveConfigPath), hf_catalog destDir.

## Research questions (для researcher-агента)

1. Спик delayed render: минимальный Rust-эксперимент WM_RENDERFORMAT (какой crate: windows-rs vs winapi vs clipboard-ext) — живой прогон на CI-раннере возможен (GUI-сессия нет) → спик ДОКУМЕНТАЛЬНЫЙ (кода-эксперимент на Windows-машине пользователя в UAT) ИЛИ CI-headless (SetClipboardData работает в headless-сервисе?). Решить: спик = план + замер протокола вставки, а не полный GUI-эксперимент.
2. cpal на WASAPI: запрашивает 16кГц mono напрямую? Если устройство отдаёт только 48кГц — нужен resample (линейная децимация достаточно? whisper чувствителен? как whisper.cpp sample-конверсия — tools/…? у нас уже есть конверсия в stt?).
3. RegisterHotKey + message-only окно: winapi CreateWindowExW MESSAGE-ONLY, цикл GetMessageW в потоке; конфликт повторной регистрации (HotKey занят → честная ошибка).
4. interprocess named pipe listener с DACL: API security_descriptor (interprocess 2.4.8 SecurityDescriptor), пример строки SDDL «D:P(A;;GA;;;%suid%)».
5. CI: расширить windows-build.yml Rust-джобой (cargo test --workspace на windows-latest) — cargo уже есть на раннере? toolchain pin?
6. cross-check с Linux: cargo check --target? (MSVC с Linux невозможен; gnu-таргет возможен, но cpal/WASAPI собирается? pragmatic: полагаемся на CI-windows-job).

## Autonomy notes

- checkpoint:human-verify задачи оркестратор разрешает сам (пользователь выдал полную автономию), фиксируя решение в отчёте.
- План ожидаемо тяжёлый (порт демона + вставка + спик + CI): split на 2 плана допустим (07-01 сервер+захват; 07-02 вставка+спик) с depends_on; решение по объёму — на чекере/оркестраторе.
- Дуга заморозки: phase-base.txt снимать с коммита исследовательского коммита фазы 7.
- UAT: 4 кейса вставки (обычное/elevated/зажатый модификатор/Chromium) — физически на Windows-машине пользователя; автоматизируемое = CI + unit-тесты + Linux-батарея.
