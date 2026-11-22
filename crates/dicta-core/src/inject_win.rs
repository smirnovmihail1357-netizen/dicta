//! Вставка текста в Windows (заглушка до 07-02).
//!
//! Production-вставка здесь ЗАПРЕЩЕНА (Must-NOT 8): спик 07-SPIKE-RESULT.md
//! обязателен до реализации. Парсер Combo уже платформо-независим и живёт в
//! `crate::combo`, поэтому 07-02 переиспользует его как есть.

use std::time::Duration;

use anyhow::Result;

pub use crate::combo::Combo;

/// На Windows нечего греть (uinput-устройства нет).
pub fn warm_up_keyboard() -> Result<()> {
    Ok(())
}

/// Честная заглушка вместо вставки. Текст НЕ логируется и НЕ сохраняется.
#[allow(clippy::too_many_arguments)]
pub fn insert_text(
    _text: &str,
    _mode: &str,
    _combo: Combo,
    _restore_after: Duration,
    _delay_before_keys: Duration,
    _atspi_allow_any_role: bool,
    _atspi_app_hint: Option<&str>,
) -> Result<()> {
    anyhow::bail!("вставка на Windows появится после спика (07-02, 07-SPIKE-RESULT.md)")
}
