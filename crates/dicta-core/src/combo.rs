//! Платформо-независимый парсер комбинации клавиш.
//!
//! Вынесен из `inject.rs` дословно, чтобы Windows-модуль вставки (`inject_win`)
//! мог переиспользовать тот же результат без uinput-зависимостей Linux.

use anyhow::Result;

const KEY_ENTER: u16 = 28;
const KEY_V: u16 = 47;
const KEY_C: u16 = 46;
const KEY_INSERT: u16 = 110;

/// Комбинация клавиш, например Ctrl+V.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Combo {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub key: u16,
}

impl Combo {
    pub fn parse(s: &str) -> Result<Self> {
        let mut combo = Combo {
            ctrl: false,
            shift: false,
            alt: false,
            key: 0,
        };
        for part in s
            .to_lowercase()
            .split('+')
            .map(str::trim)
            .filter(|p| !p.is_empty())
        {
            match part {
                "ctrl" | "control" => combo.ctrl = true,
                "shift" => combo.shift = true,
                "alt" => combo.alt = true,
                "v" => combo.key = KEY_V,
                "c" => combo.key = KEY_C,
                "insert" | "ins" => combo.key = KEY_INSERT,
                "enter" | "return" => combo.key = KEY_ENTER,
                other => anyhow::bail!("неизвестная клавиша {other:?} в combos"),
            }
        }
        if combo.key == 0 {
            anyhow::bail!("в комбинации {s:?} нет основной клавиши");
        }
        Ok(combo)
    }
}
