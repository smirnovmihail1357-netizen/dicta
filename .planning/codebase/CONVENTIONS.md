---
last_mapped_commit: 20e00408af8921966d7d02f862d064e71658eb28
last_mapped_at: 2026-11-22
---
# Coding Conventions

**Analysis Date:** 2026-11-22

## Общий стиль

**Язык проекта:** русский. Все комментарии, doc-комментарии, UI-тексты, логи,
коммит-сообщения и сообщения об ошибках пишутся по-русски. Идентификаторы, имена
файлов и конфиг-ключи — английские. Пример из `crates/dicta-core/src/filter.rs`:

```rust
/// Whisper на тишине/шуме часто изобретает готовые фразы. Убираем их.
const HALLUCINATIONS: &[&str] = &[ ... ];
```

## Naming Patterns

**Files:**
- snake_case, один модуль = один файл: `crates/dicta-core/src/paste_delay` нет — есть `inject.rs`, `focus.rs`, `hotkey.rs`, `atspi.rs`, `capture.rs`, `filter.rs`, `audio.rs`, `config.rs`, `stt.rs`.
- Тесты живут в том же файле в `#[cfg(test)] mod tests` — отдельных `tests/` директорий нет.

**Crates:**
- Имена бинарников: `dictad` (демон), `dictactl` (CLI-клиент), `dicta-ui` (оверлей), `dicta-settings` (окно настроек); библиотека — `dicta-core`. Пакеты лежат в `crates/<имя>`.

**Functions:**
- snake_case; короткие глагольные имена: `clean`, `looks_repetitive`, `normalize`, `write_focus`, `read_focus` (`crates/dicta-core/src/filter.rs`, `crates/dicta-core/src/focus.rs`).
- Хелперы-конструкторы данных для тестов: `fn noise(amp: i16, n: usize) -> Vec<i16>` (`crates/dicta-core/src/audio.rs`), `fn models() -> Vec<String>` (`crates/dictad/src/main.rs`).

**Variables:**
- snake_case; верхний регистр `SCREAMING_CASE` для констант: `DRAG_THRESHOLD`, `HALLUCINATIONS`, `SURF_W`, `REST_GAP`.

**Types:**
- CamelCase: `Config`, `Placement`, `Edge`, `World`, `Release`, `HfRepo`.
- Enum-варианты описывают действие/результат по-русски в doc, но идентификаторы английские: `Release::Click`, `Release::Dock(Placement)`, `Change::Text(_, _)`, `Change::Level(_, _)`.

**Config-ключи (TOML и сокет-протокол):**
- snake_case, совпадает с полями `Config` в `crates/dicta-core/src/config.rs`: `min_level`, `paste_delay_ms`, `inject_mode`, `use_gpu`, `paste_keys_by_app`.

## Code Style

**Formatting:**
- `cargo fmt` обязателен (см. `CONTRIBUTING.md`); отдельного `rustfmt.toml` нет — стиль дефолтный.
- Строки длиннее дефолтных 100 разрешены в тестах и doc-комментариях (фактически не принудительно).

**Linting:**
- Clippy в CI не гоняется; CI (`.github/workflows/ci.yml`) проверяет `cargo build --release --locked` (Vulkan и CPU-only варианты) + `cargo test --workspace` + `bash -n` для шелл-скриптов.

**Edition:** Rust 2021, `resolver = "2"`, `[workspace.package] version = "0.3.0"`.

## Import Organization

**Порядок:** (фактически — std → внешние крейты → свои модули, как принято в rustfmt):
1. `std::...`
2. Внешние крейты (`anyhow`, `regex`, `serde`, `clap`, `dicta_core`)
3. `crate::...` / `super::*` в тестах

```rust
// crates/dicta-core/src/config.rs
use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;
```

**Path Aliases:** нет;跨-crate импорт через имя библиотеки `dicta_core` (например `use dicta_core::{capture, load_wav, resample_to_16k, Config, Engine};` в `crates/dictactl/src/main.rs`).

## Doc-комментарии (`//!` и `///`)

**Module-level `//!` — обязательный паттерн.** Почти каждый модуль открывается
русским `//!`-блоком, объясняющим задачу модуля и ключевые design-решения:

```rust
// crates/dicta-ui/src/drag.rs
//! Машина перетаскивания пилюли: чистое состояние без Wayland.
//!
//! Живёт отдельно от Ui, чтобы сценарии перетаскивания можно было проигрывать
//! синтетическими событиями мыши в юнит-тестах — «зажал у нижней грани,
//! протащил к верхней, отпустил» — без композитора и живого сеанса.
```

Другие примеры: `crates/dicta-core/src/inject.rs`, `crates/dicta-core/src/hotkey.rs`,
`crates/dicta-core/src/focus.rs` (нумерованная схема фолбэков), `crates/dicta-core/src/atspi.rs`.

**Item-level `///` — на каждом pub-элементе**, по-русски, часто с обоснованием
«почему именно так» и замерами с реальной машины:

```rust
// crates/dicta-core/src/config.rs
/// Минимальный уровень записи в той же шкале 0..1, что и индикатор в UI ...
/// Измерено на этой машине: фон в комнате 0.34-0.35, русская речь 0.9-1.0.
pub min_level: f32,
```

## Error Handling

**Стратегия:** `anyhow` везде, где сбой пробрасывается вверх; конкретный
`Result<T, String>` — на границе сокет-протокола, где текст ошибки уходит
клиенту в JSON-поле `error`.

**Паттерны:**
- `use anyhow::{Context, Result}` + `.context(...)` при чтении конфига (`crates/dicta-core/src/config.rs`).
- Валидация сетевых команд демона: `Result<Change, String>` — ошибки валидации по-русски: `"min_level: 2 вне диапазона 0..1"` (`crates/dictad/src/main.rs:497 parse_change`, тест `values_are_validated_before_they_touch_the_config`).
- **Никаких молчаливых падений** (правило из `CONTRIBUTING.md`): сбой логируется `eprintln!("dictad: ...")` в stderr демона (уходит в `journalctl --user -u dictad.service`) И попадает в поле `error` ответа сокета.
- Префикс логов демона: `dictad: ` (`crates/dictad/src/main.rs:194,257,262,279`).
- Паники с русским сообщением допустимы только в тестах: `panic!("ожидался Text, получили {other:?}")`.
- Проверка вместо слепого sleep: «вставка без лишних задержек: inference в воркере, restore буфера в фоне, проверка вместо слепого sleep» (коммит `f48c12a`).

## Logging

**Framework:** `eprintln!` в stderr демона — без tracing/log-крейтов. Метод
`.elapsed()` через `std::time::Instant` для таймингов загрузки модели
(`crates/dictad/src/main.rs:771`).

**Patterns:**
- Каждое существенное событие демона: загрузка модели, готовность uinput, авто-остановка записи, ошибки вставки.
- UI-диагностика — тоже в stderr, читается через journalctl.

## Comments

**When to Comment:**
- Объяснение design-решения и его причины, а не пересказ кода: «порог втягивания: движение меньше этого — ещё клик, не перетаскивание» (`crates/dicta-ui/src/drag.rs`).
- Предостережения о среде: Vulkan-нумерация ggml ≠ `/sys/class/drm/cardN` (`CONTRIBUTING.md`), Wayland surface становится mapped только на commit с буфером.
- В тестах — входные данные сценария: «2 с фона + 0.3 с "речи" в середине» (`crates/dicta-core/src/audio.rs`).

**JSDoc/TSDoc → rustdoc:** см. раздел «Doc-комментарии» выше.

## Function Design

**Size:** небольшие функции, часто < 40 строк; каждый слайдер-хелпер в `crates/dicta-settings/src/main.rs` — отдельная функция с doc-комментарием.

**Parameters:** маленькие POD-структуры вместо россыпи аргументов: `World { placement, window, screen }` в `crates/dicta-ui/src/drag.rs`.

**Return Values:** `Result` из anyhow; чистые парсеры возвращают `Option`/значение: `Edge::parse(&str) -> Option<Edge>`, `normalize(&str) -> String`.

## Module Design

**Exports:** минимальный `pub use` в `crates/dicta-core/src/lib.rs` — только то, что нужно бинарникам (`pub use audio::{load_wav, normalized_rms, resample_to_16k, silence_streak_ms, speech_level};`).

**Barrel Files:** нет; модули плоские в `src/`, подключены `pub mod` в `lib.rs`.

**Чистое ядро для тестов:** интерактивная логика выделяется в отдельный файл
без привязок к ОС, чтобы тестировать синтетикой: `crates/dicta-ui/src/drag.rs`
(«машина перетаскивания как чистый компонент», `Edge`/`Placement` в
`crates/dicta-ui/src/edge.rs`, `paint()` в `crates/dicta-ui/src/main.rs`).

## Слайдеры egui: «send on drag_stopped»

В `crates/dicta-settings/src/main.rs` все слайдеры (`level_slider` :956,
`ms_slider` :981, `delay_slider` :994, `int_slider` :1010) следуют одному
паттерну — значение отправляется демону **по отпусканию ползунка, а не на
каждый кадр**:

```rust
fn level_slider(ui: &mut egui::Ui, app: &mut App, key: &'static str, label: &str, value: f32) {
    let mut pending = value;
    let response = ui.add(
        egui::Slider::new(&mut pending, 0.05..=1.0)
            .text(label)
            .step_by(0.05)
            .custom_formatter(|v, _| format!("{v:.2}")),
    );
    if response.drag_stopped() || (response.changed() && !ui.ctx().is_using_pointer()) {
        app.set(key, format!("{pending:.2}"));
    }
}
```

- `let mut pending = value` — локальная копия, живой `value` не мутируется.
- Условие `drag_stopped() || (changed() && !is_using_pointer())` покрывает и drag, и клик/клавиатурное изменение.
- Показ — человекочитаемый (`custom_formatter`: секунды вместо мс), отправка — в родных единицах демона.
- Чекбоксы шлют нативные `"on"/"off"` демона (`checkbox`, `crates/dicta-settings/src/main.rs:1018`).
- Новый слайдер = скопировать один из четырёх хелперов, заменить `key`, диапазон и форматтер.

## Стиль коммит-сообщений (русский, lowercase, без префиксов типов)

По `git log` — сплошной стиль:

```
20e0040 браузер моделей HF: фильтры задачи и сортировки, пагинация, подпапки, VRAM-лимит
f48c12a вставка без лишних задержек: inference в воркере, restore буфера в фоне, проверка вместо слепого sleep
d4c0d72 fix: буфер ореола до первого configure — protocol error и смерть UI
6b2dd04 ci: libxkbcommon-dev в сборочные образы (build-скрипт smithay-client-toolkit требует xkbcommon через pkg-config)
2e4c041 docs: RELEASE.md приведён к выложенному репозиторию ...
```

Правила:
- **Строчная буква в начале, по-русски**, тема — что изменилось в поведении пользователя/системы.
- Формат «`<область>: <что и почему>`» с двоеточием: `окно настроек:`, `drag.rs:`, `gpu_device = 0:`.
- Префиксы `fix:`/`ci:`/`docs:` — редкие, английские, когда фикс/инфраструктура.
- Тело через тире, объяснение причины прямо в теме: «(build-скрипт ... требует xkbcommon через pkg-config)», «нумерация Vulkan проверена журналом, 0 — быстрое (~200 мс)».
- Никаких conventional-commits полей (нет `feat:`, `BREAKING CHANGE`, issue-ссылок).

## Протокол сокета (обратно-совместимость)

Одна JSON-строка в одну сторону (`crates/dictad/src/main.rs`, `crates/dicta-settings/src/net.rs`):

```json
{"cmd":"toggle","app":"konsole","text":"...","keys":"ctrl+shift+v"}
-> {"state":"recording","level":0.42,"seconds":1.8,"text":null,"error":null}
```

- Неизвестные поля игнорируются; новые команды добавляются без смены версии.
- Ошибка — всегда текст в поле `error`, по-русски; новых полей достаточно, версионизации нет.

## Прочие обязательные правила (из CONTRIBUTING.md)

- Новые зависимости — только при реальной необходимости (whisper-rs уже тащит whisper.cpp).
- GPU: модель одна и резидентная; никаких параллельных whisper/llama-экземпляров.
- Сборка CPU-варианта (`--no-default-features`) не должна ломаться.
- `dicta-ui` обязан присоединять буфер и в `idle` — иначе Wayland не даст frame-колбэков.

---

*Convention analysis: 2026-11-22*
