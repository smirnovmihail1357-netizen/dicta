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
];

fn patterns() -> &'static [Regex] {
    static CACHE: OnceLock<Box<[Regex]>> = OnceLock::new();
    CACHE.get_or_init(|| {
        HALLUCINATIONS
            .iter()
            .filter_map(|p| {
                RegexBuilder::new(p)
                    .case_insensitive(true)
                    .build()
                    .ok()
            })
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
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
