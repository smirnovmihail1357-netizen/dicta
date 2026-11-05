use std::sync::OnceLock;

use regex::{Regex, RegexBuilder};

/// Whisper на тишине/шуме часто изобретает готовые фразы. Убираем их.
const HALLUCINATIONS: &[&str] = &[
    r"^\[.*\]$",
    r"^\(.*\)$",
    r"^(thanks|thank you) (for watching|for listening)[.!]*$",
    r"^you[.!]*$",
    r"^субтитры (сделал|создавал|подогнал).*$",
    r"^редактор субтитров.*$",
    r"^продолжение следует\.?$",
    r"^спасибо за просмотр[.!]*$",
    r"^по-русски([,. ]*по-русски)*[.!]*$",
    r"^(subtitles|субтитры) (by|от) .*$",
    r"^www\.[a-z0-9.-]+$",
    r"^подписал(ись)? на канал.*$",
];

/// Текст, собранный из повторений одной короткой фразы — типичный галлюцинаторный
/// выход whisper на шуме («По-русски, по-русски, по-русски.», «спасибо за просмотр» x3).
pub fn looks_repetitive(text: &str) -> bool {
    let words: Vec<String> = text
        .to_lowercase()
        .split(|c: char| c.is_whitespace() || c == ',' || c == '.')
        .filter(|w| !w.is_empty())
        .map(|w| w.to_string())
        .collect();
    if words.len() < 3 {
        return false;
    }
    for n in 1..=3usize {
        let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for w in words.windows(n) {
            *seen.entry(w.join(" ")).or_insert(0) += 1;
        }
        let repeated: usize = seen.values().filter(|c| **c >= 3).sum();
        if repeated * n >= words.len() / 2 {
            return true;
        }
    }
    false
}

fn patterns() -> &'static [Regex] {
    static CACHE: OnceLock<Box<[Regex]>> = OnceLock::new();
    CACHE.get_or_init(|| {
        HALLUCINATIONS
            .iter()
            .filter_map(|p| RegexBuilder::new(p).case_insensitive(true).build().ok())
            .collect::<Vec<Regex>>()
            .into_boxed_slice()
    })
}

/// Склеивает сегменты в одну строку, выбрасывая галлюцинации и пустое.
pub fn clean(text: &str) -> String {
    let re = patterns();
    text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .filter(|l| !re.iter().any(|r| r.is_match(l)))
        .filter(|l| !looks_repetitive(l))
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuts_known_artifacts() {
        assert_eq!(clean("[Музыка]").trim(), "");
        assert_eq!(clean("спасибо за просмотр"), "");
        assert_eq!(clean("По-русски, по-русски, по-русски."), "");
        assert_eq!(clean("редактор субтитров"), "");
    }

    #[test]
    fn keeps_real_speech() {
        assert_eq!(
            clean("Rust, TypeScript, JSON, API, git, PipeWire."),
            "Rust, TypeScript, JSON, API, git, PipeWire."
        );
        assert_eq!(
            clean("надо перезапустить демон и проверить вставку в konsole"),
            "надо перезапустить демон и проверить вставку в konsole"
        );
    }

    #[test]
    fn repetition_detector() {
        assert!(looks_repetitive("по-русски по-русски по-русски"));
        assert!(looks_repetitive(
            "спасибо за просмотр спасибо за просмотр спасибо за просмотр"
        ));
        assert!(!looks_repetitive(
            "надо перезапустить демон и проверить вставку"
        ));
    }
}
