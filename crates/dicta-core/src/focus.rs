//! Как демон узнаёт, в какое окно вставлять текст.
//!
//! Композиторы не дают единого «спросить фокусное окно» способа: у KWin метод
//! `queryWindowInfo` интерактивный (показывает оверлей «укажите окно»), поэтому
//! спрашивать его из демона нельзя. Схема такая:
//!
//! 1. адаптер знает своё фокусное окно и передаёт его демону вместе с командой:
//!    `dictactl toggle --app konsole` (KWin-скрипт, Hyprland `$focused`, sway);
//! 2. либо фокус складывается в кэш-файл `~/.cache/dicta/focus` (его пишет любой
//!    скрипт слежения за фокусом; см. `dictactl focus-report`);
//! 3. если ничего не известно — используются `paste_keys` из конфига.

use std::path::PathBuf;

/// Файл, куда внешние скрипты складывают имя фокусного приложения.
pub fn focus_cache_path() -> PathBuf {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".cache")
        });
    base.join("dicta").join("focus")
}

/// Записать фокусное приложение (вызывает адаптер: `dictactl focus-report konsole`).
pub fn write_focus_cache(app: &str) -> std::io::Result<()> {
    write_focus(&focus_cache_path(), app)
}

pub fn write_focus(path: &std::path::Path, app: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, app)
}

pub fn read_focus(path: &std::path::Path) -> Option<String> {
    let raw = std::fs::read_to_string(path).ok()?;
    let app = raw.trim().to_string();
    (!app.is_empty()).then_some(app)
}

/// Имя приложения из кэша фокуса (None, если файла нет или он пуст).
pub fn focus_from_cache() -> Option<String> {
    read_focus(&focus_cache_path())
}

/// Класс окна для подбора клавиш: срезем `.desktop`, затем всё после последней
/// точки (`firefox.desktop` -> `firefox`, `org.kde.konsole` -> `konsole`).
pub fn normalize(app: &str) -> String {
    let base = app.strip_suffix(".desktop").unwrap_or(app);
    base.rsplit('.')
        .next()
        .unwrap_or(base)
        .to_ascii_lowercase()
        .replace(['-', '_'], "")
}

/// Имя приложения для текущей вставки: подсказка адаптера важнее кэша.
pub fn active_app(hint: Option<&str>) -> Option<String> {
    match hint {
        Some(hint) if !hint.trim().is_empty() => Some(hint.trim().to_string()),
        _ => std::env::var("DICTA_APP")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .or_else(focus_from_cache),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_focus_file() {
        let path = std::env::temp_dir().join("dicta-test-focus");
        write_focus(&path, "org.kde.konsole").unwrap();
        assert_eq!(read_focus(&path).as_deref(), Some("org.kde.konsole"));
        std::fs::remove_file(&path).ok();
        assert_eq!(read_focus(&path), None);
    }

    #[test]
    fn normalized() {
        assert_eq!(normalize("org.kde.konsole"), "konsole");
        assert_eq!(normalize("firefox.desktop"), "firefox");
        assert_eq!(normalize("Waybar"), "waybar");
        // ключи в paste_keys_by_app надо писать уже нормализованными:
        // telegram-desktop и telegramdesktop -> telegramdesktop
        assert_eq!(normalize("telegram-desktop"), "telegramdesktop");
        assert_eq!(normalize("Mozilla Firefox"), "mozilla firefox");
    }

    #[test]
    fn hint_wins() {
        assert_eq!(active_app(Some("kate")).as_deref(), Some("kate"));
    }
}
