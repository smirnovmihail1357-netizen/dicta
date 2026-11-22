//! Каталог ggml-моделей whisper.cpp и чистые функции путей для скачивания.
//! Никакой работы с сетью и сокетом здесь нет — только строки и `PathBuf`.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Одна строка каталога: имя файла, пояснение и примерный размер для прогресса.
pub struct CatalogEntry {
    pub file: &'static str,
    pub note: &'static str,
    pub approx_bytes: u64,
}

/// Модели в порядке от быстрой к качественной (тот же ряд, что в ollama/whisper.cpp).
pub const CATALOG: [CatalogEntry; 6] = [
    CatalogEntry {
        file: "ggml-tiny-q5_1.bin",
        note: "сверхбыстрая, для теста",
        approx_bytes: 32_000_000,
    },
    CatalogEntry {
        file: "ggml-base-q5_1.bin",
        note: "быстрая, простые фразы",
        approx_bytes: 60_000_000,
    },
    CatalogEntry {
        file: "ggml-small-q5_1.bin",
        note: "баланс скорости и качества",
        approx_bytes: 180_000_000,
    },
    CatalogEntry {
        file: "ggml-medium-q5_0.bin",
        note: "хорошее качество",
        approx_bytes: 500_000_000,
    },
    CatalogEntry {
        file: "ggml-large-v3-turbo-q5_1.bin",
        note: "лучшее качество при высокой скорости",
        approx_bytes: 570_000_000,
    },
    CatalogEntry {
        file: "ggml-large-v3-q5_0.bin",
        note: "максимум качества, медленнее",
        approx_bytes: 1_100_000_000,
    },
];

/// Прямая ссылка на файл в репозитории whisper.cpp (как в tools/fetch-model.sh).
pub fn url_for(file: &str) -> String {
    format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{file}")
}

/// Незаконченный файл: тот же путь с расширением `.part` (докачка через curl -C -).
pub fn part_of(dest: &Path) -> PathBuf {
    let mut part = dest.as_os_str().to_os_string();
    part.push(".part");
    PathBuf::from(part)
}

/// Куда кладём скачанную модель: рядом с текущей, чтобы она сразу появилась
/// в `settings.models` демона (там читается parent текущей модели).
pub fn dest_dir(model: &str) -> PathBuf {
    dest_dir_from(
        model,
        std::env::var_os("XDG_DATA_HOME"),
        std::env::var_os("HOME"),
    )
}

/// Чистая версия `dest_dir` для тестов: без чтения окружения процесса.
fn dest_dir_from(model: &str, xdg_data_home: Option<OsString>, home: Option<OsString>) -> PathBuf {
    let parent = Path::new(model)
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty());
    match parent {
        Some(dir) => dir.to_path_buf(),
        None => {
            // Пустое имя или имя без каталога — та же папка, что в fetch-model.sh.
            let base = match xdg_data_home.filter(|v| !v.is_empty()) {
                Some(dir) => PathBuf::from(dir),
                None => PathBuf::from(home.unwrap_or_default())
                    .join(".local")
                    .join("share"),
            };
            base.join("whisper-cpp")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_for_builds_correct_link() {
        assert_eq!(
            url_for("ggml-tiny-q5_1.bin"),
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny-q5_1.bin"
        );
        assert!(url_for("ggml-x.bin").contains("/ggerganov/whisper.cpp/resolve/main/"));
    }

    #[test]
    fn part_of_appends_extension() {
        assert_eq!(
            part_of(Path::new("/a/b/ggml-x.bin")),
            PathBuf::from("/a/b/ggml-x.bin.part")
        );
        assert_eq!(
            part_of(Path::new("ggml-x.bin")),
            PathBuf::from("ggml-x.bin.part")
        );
    }

    #[test]
    fn dest_dir_uses_parent_of_current_model() {
        assert_eq!(
            dest_dir_from("/a/b/ggml-x.bin", None, Some("/home/u".into())),
            PathBuf::from("/a/b")
        );
    }

    #[test]
    fn dest_dir_falls_back_to_data_home() {
        let expected = PathBuf::from("/home/u/.local/share/whisper-cpp");
        assert_eq!(dest_dir_from("", None, Some("/home/u".into())), expected);
        assert_eq!(
            dest_dir_from("ggml-x.bin", None, Some("/home/u".into())),
            expected
        );
    }

    #[test]
    fn dest_dir_honors_xdg_data_home() {
        assert_eq!(
            dest_dir_from("", Some("/data".into()), Some("/home/u".into())),
            PathBuf::from("/data/whisper-cpp")
        );
    }

    #[test]
    fn catalog_is_ordered_and_sized() {
        assert_eq!(CATALOG.len(), 6);
        assert_eq!(CATALOG[0].file, "ggml-tiny-q5_1.bin");
        assert_eq!(CATALOG[5].file, "ggml-large-v3-q5_0.bin");
        for entry in &CATALOG {
            assert!(entry.file.starts_with("ggml-"));
            assert!(entry.file.ends_with(".bin"));
            assert!(entry.approx_bytes > 0);
            assert!(!entry.note.is_empty());
        }
    }
}
