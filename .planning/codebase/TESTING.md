---
last_mapped_commit: 20e00408af8921966d7d02f862d064e71658eb28
last_mapped_at: 2026-11-22
---
# Testing Patterns

**Analysis Date:** 2026-11-22

## Test Framework

**Runner:**
- Встроенный `cargo test` (libtest). Отдельного test-фреймворка и dev-dependencies нет — ноль внешних тестовых крейтов.

**Assertion Library:**
- Стандартный макрос `assert!` / `assert_eq!`, без `pretty_assertions` и подобных.

**Run Commands:**
```bash
cargo test --workspace          # все тесты (Makefile: make test)
cargo test -p dicta-ui          # тесты одного крейта
```

CI (`.github/workflows/ci.yml`) гоняет `cargo test --workspace` в контейнере `rust:1-trixie` после сборки Vulkan и CPU-only вариантов.

## Test File Organization

**Location:** все тесты — инлайновые unit-тесты, co-located с кодом в том же
файле внутри `#[cfg(test)] mod tests { use super::*; ... }`. Директории
`tests/` (integration) не используются.

**Naming:**
- Тест-функции — английские snake_case, описывают поведение: `cuts_known_artifacts`, `keeps_real_speech`, `default_placement_is_the_old_bottom_pill`, `values_are_validated_before_they_touch_the_config`.
- Файл = модуль = свои тесты в его конце: `crates/dicta-core/src/filter.rs:72` и далее.

**Structure / распределение по крейтам** (число функций `#[test]` на момент анализа):

| Crate | Тестов | Где |
|-------|--------|-----|
| `dicta-core` | 15 | `config.rs` (5), `filter.rs` (3), `audio.rs` (3), `focus.rs` (3), `atspi.rs` (1) |
| `dicta-ui` | 42 | `edge.rs` (22), `main.rs` (8), `drag.rs` (7), `panel.rs` (5) |
| `dicta-settings` | 25 | `hf.rs` (10), `catalog.rs` (6), `main.rs` (4), `state.rs` (3), `net.rs` (2) |
| `dictad` | 3 | `main.rs` |
| `dictactl` | 0 | — |

Итого ≈ 85 unit-тестов.

## Test Structure

**Suite Organization** — канонический паттерн каждого модуля:

```rust
// crates/dicta-core/src/filter.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuts_known_artifacts() {
        assert_eq!(clean("[Музыка]").trim(), "");
        assert_eq!(clean("спасибо за просмотр"), "");
        assert_eq!(clean("По-русски, по-русски, по-русски."), "");
    }

    #[test]
    fn keeps_real_speech() { /* речь не должна пострадать от фильтра */ }
}
```

**Patterns:**
- Позитивные и негативные тесты рядом: «мусор вырезается» + «настоящая речь сохраняется» (`filter.rs`).
- Хелперы данных в начале `mod tests`: `fn noise(amp: i16, n: usize) -> Vec<i16>` (`audio.rs`), `fn models() -> Vec<String>` (`dictad/main.rs`).
- Комментарий-сценарий перед входными данными: `// 2 с фона + 0.3 с «речи» в середине` (`crates/dicta-core/src/audio.rs`).
- Формат `assert!(cond, "сообщение по-русски: {value}")` — сообщение поясняет, что именно сломалось (`audio.rs`, `main.rs` dicta-ui: `"пружина должна быть заметна"`).
- Ветвление по варианту enum — с паникой-ловлей: `other => panic!("ожидался Text, получили {other:?}")` (`dictad/main.rs:1315`, `drag.rs:296`).

## Mocking

**Framework:** нет. Моки не используются — **это принцип проекта**: тесты unit,
без микрофона и без GPU (`CONTRIBUTING.md`: «`cargo fmt` и `cargo test` должны
проходить; тесты — unit, без микрофона и без GPU»).

Вместо моков — два приёма:

1. **Чистое ядро без ОС-привязок.** Интерактивные компоненты выделены так,
   чтобы тестировать их синтетикой: `crates/dicta-ui/src/drag.rs` — «машина
   перетаскивания пилюли: чистое состояние без Wayland ... проигрывать
   синтетическими событиями мыши в юнит-тестах без композитора и живого
   сеанса». Ui остаётся только доставка событий и рисование.
2. **Парсеры принимают строки вместо сети/устройств.** `parse_search(json: &str)`, `parse_tree(json: &str)` в `crates/dicta-settings/src/hf.rs` вызываются на захардкоженных JSON-строках; `speech_level(&[i16])` в `audio.rs` — на синтетическом шуме.

**What NOT to Mock:** сеть, Wayland, PipeWire, whisper — вместо мока логика
разнесена на «чистую функцию + тонкий обвяз» и тестируется чистая функция.

## Fixtures and Factories

**Test Data:** строковые литералы прямо в тестах.

- JSON-сэмплы Hugging Face API — сырые строки `r#"[...]"#` в `crates/dicta-settings/src/hf.rs` (тесты `parse_search_reads_id_downloads_and_likes`, `parse_tree_keeps_only_bin_files_and_uses_lfs_size`) — **без сети**, включая кривые входы: `{"id":17}`, `null`-поля, `"likes":"12"`.
- `serde_json::json!`-макрос для ответов демона в `crates/dicta-settings/src/state.rs` (`from_reply_reads_settings_and_error`), включая тест «старый демон без поля settings»: `json!({"base": {"error": null}})`.
- Синтетические аудиосэмплы: меандр `noise(100, 16000*2)` с вживлённым «речевым» всплеском через `splice` (`audio.rs`).
- Файл на диске для файловых функций: `std::env::temp_dir().join("dicta-test-focus")` + удаление в конце (`focus.rs`, `parse_focus_file`).

**Location:** fixtures не вынесены — живут внутри `mod tests` каждого файла.

## Coverage

**Requirements:** численного порога нет; критерий — `cargo test --workspace` зелёный (CI) + ручные проверки из `CONTRIBUTING.md`:

```bash
./tools/insert-test.sh          # живая вставка в окно (нужен foot)
dictactl calibrate --secs 8     # пороги уровня на реальном микрофоне
dictactl atspi-dump --depth 8   # дерево доступности
journalctl --user -u dictad.service -f
```

**View Coverage:** `cargo tarpaulin`/`llvm-cov` не подключены.

**Принцип полноты:** для каждого чистого компонента — плотный набор сценариев
(у `edge.rs` — 22 теста на разбор сторон, круговое переключение, маргины,
вертикальные перестановки; у `drag.rs` — сценарии «зажал-протащил-отпустил»
на все четыре грани).

## Test Types

**Unit Tests:**
- Основной и практически единственный тип: парсеры, фильтры, геометрия,
  state-машины, валидация команд, рендер пикселей. Без сети, без GPU, без
  Wayland — `cargo test` запускается на любой машине без зависимостей среды.

**Integration / E2E:**
- Формально отсутствуют. Ручные сценарии: скриншотная проверка оверлея
  (`spectacle -b -n -o /tmp/s.png` и сравнение полосы, `CONTRIBUTING.md`),
  живая вставка `tools/insert-test.sh`.

**Рендер-тесты как unit:** `crates/dicta-ui/src/main.rs` рисует кадры в
программный canvas функцией `paint(...)` и проверяет пиксели: `hidden_frame_draws_nothing`
(все байты нули), `resting_frame_draws_pill`, `vertical_frame_draws_a_turned_pill`,
`panel_frame_draws_its_labels` — «скриншот-тесты» без композитора.

## Common Patterns

**Parser Testing (строковые сэмплы, без сети):**
```rust
// crates/dicta-settings/src/hf.rs
#[test]
fn parse_search_reads_id_downloads_and_likes() {
    let json = r#"[
        {"id":"ggerganov/whisper.cpp","downloads":12345,"likes":678,...},
        {"id":"old/style/repo","likes":null},
        {"id":17,"downloads":900,"likes":5}
    ]"#;
    assert_eq!(parse_search(json), vec![ /* только валидные записи, нули вместо null */ ]);
}
```

Тестируются одновременно: happy path, пропуски полей, битые типы, лишние поля.

**Точная проверка строк-протоколов:**
```rust
// crates/dicta-settings/src/net.rs
assert_eq!(settings_json(), "{\"cmd\":\"settings\"}\n");
assert_eq!(set_json("model", "C:\\models\\ggml-\"x\".bin"),
    "{\"cmd\":\"set\",\"key\":\"model\",\"value\":\"C:\\\\models\\\\ggml-\\\"x\\\".bin\"}\n");
```
— экранирование JSON проверяется байт-в-байт.

**Сценарные тесты state-машины:**
```rust
// crates/dicta-ui/src/drag.rs — синтетические события мыши
// «зажал у нижней грани, протащил к верхней, отпустил»
other => panic!("ожидали закрепление сверху, получили {other:?}"),
```

**Error Testing:**
```rust
// crates/dictad/src/main.rs
assert!(parse_change("min_level", "4", &models()).is_err());
assert!(parse_change("inject_mode", "telepathy", &models()).is_err());
assert!(parse_change("weather", "sunny", &models()).is_err());   // неизвестный ключ
```

**Числа с плавающей точкой:**
```rust
assert!(matches!(
    parse_change("min_level", "0.4", &models()).unwrap(),
    Change::Level(_, v) if (v - 0.4).abs() < 1e-6
));
assert!((settings.min_level - 0.4).abs() < f32::EPSILON);   // state.rs
```

**Диапазонные/инвариантные проверки вместо точных значений:**
```rust
// crates/dicta-ui/src/main.rs
let overshoot_px = (peak - 1.0) * render::TRAVEL as f32;
assert!(overshoot_px < render::HEADROOM as f32,
    "overshoot {overshoot_px}px не должен обрезаться краем поверхности");
```

## Правила для новых тестов (прескриптивно)

1. Тест пишется в тот же файл под `#[cfg(test)] mod tests` с `use super::*;` — не в отдельный `tests/` crate.
2. Ничего внешнего: без сети (вход — строковый сэмпл), без микрофона, без GPU, без Wayland. Если тестируемый код так не умеет — вынести логику в чистую функцию (паттерн `drag.rs`).
3. Имя теста — поведение: `<subject>_<does_what>` (`edge_parses_four_sides_and_labels`, `model_rejects_what_is_not_there`).
4. Негативные входы обязательны: битый JSON, null, неизвестные ключи, выход из диапазона.
5. Сообщение `assert!` — по-русски, со значениями в `{}`.
6. Плавающие — через `(a - b).abs() < eps`, диапазоны — через `assert!(min < x < max)`.
7. Новая ветка enum — `other => panic!("ожидали ..., получили {other:?}")`.

---

*Testing analysis: 2026-11-22*
