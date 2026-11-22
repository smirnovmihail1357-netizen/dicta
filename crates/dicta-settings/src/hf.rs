//! Чистые функции Hugging Face API: URL, парсеры поиска и дерева файлов.
//! Никакой работы с сетью и сокетом — только строки и JSON, всё тестируемо без интернета.

/// Percent-encoding: всё вне `[A-Za-z0-9-._~]` кодируется байтом в `%XX`.
pub fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &byte in s.as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// Поиск моделей, отсортированный по числу скачиваний.
pub fn search_url(query: &str) -> String {
    format!(
        "https://huggingface.co/api/models?search={}&sort=downloads&direction=-1&limit=20",
        url_encode(query)
    )
}

/// Корень репозитория (без рекурсии: нас интересуют файлы верхнего уровня).
pub fn tree_url(repo: &str) -> String {
    format!("https://huggingface.co/api/models/{repo}/tree/main?recursive=false")
}

/// Прямая ссылка на файл репозитория для `curl -L`.
pub fn resolve_url(repo: &str, path: &str) -> String {
    format!("https://huggingface.co/{repo}/resolve/main/{path}")
}

/// Строка результата поиска: идентификатор репозитория и число скачиваний.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfRepo {
    pub id: String,
    pub downloads: u64,
}

/// Разбор ответа `/api/models?search=...`: битый JSON и не-массив дают пустой список.
pub fn parse_search(json: &str) -> Vec<HfRepo> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return Vec::new();
    };
    let Some(items) = value.as_array() else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            let id = item
                .get("id")
                .or_else(|| item.get("modelId"))
                .and_then(|v| v.as_str())?;
            let downloads = item.get("downloads").and_then(|v| v.as_u64()).unwrap_or(0);
            Some(HfRepo {
                id: id.to_string(),
                downloads,
            })
        })
        .collect()
}

/// Файл репозитория: путь внутри него и размер в байтах (0 — размер неизвестен).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfFile {
    pub path: String,
    pub size: u64,
}

/// Разбор ответа `/api/models/{repo}/tree/main`: только обычные `.bin`-файлы.
pub fn parse_tree(json: &str) -> Vec<HfFile> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return Vec::new();
    };
    let Some(items) = value.as_array() else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            if item.get("type").and_then(|v| v.as_str()) != Some("file") {
                return None;
            }
            let path = item.get("path").and_then(|v| v.as_str())?;
            if !path.ends_with(".bin") {
                return None;
            }
            // У LFS-файлов размер может лежать в `lfs.size`, а `size` быть нулём.
            let size = item
                .get("size")
                .and_then(|v| v.as_u64())
                .filter(|&n| n > 0)
                .or_else(|| {
                    item.get("lfs")
                        .and_then(|lfs| lfs.get("size"))
                        .and_then(|v| v.as_u64())
                })
                .unwrap_or(0);
            Some(HfFile {
                path: path.to_string(),
                size,
            })
        })
        .collect()
}

/// Имя файла из прямой ссылки: последний сегмент пути без query-части.
pub fn filename_of(url: &str) -> Option<String> {
    let path = url.split('?').next().unwrap_or(url);
    if !path.contains('/') {
        return None;
    }
    let name = path.rsplit('/').next().unwrap_or("");
    if name.is_empty() {
        return None;
    }
    Some(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_encode_escapes_everything_outside_the_unreserved_set() {
        assert_eq!(url_encode("whisper ggml"), "whisper%20ggml");
        assert_eq!(url_encode("aa/bb"), "aa%2Fbb");
        assert_eq!(url_encode("A-z0._~"), "A-z0._~");
        assert_eq!(url_encode("ключ"), "%D0%BA%D0%BB%D1%8E%D1%87");
    }

    #[test]
    fn api_urls_are_exact() {
        assert_eq!(
            search_url("whisper ggml"),
            "https://huggingface.co/api/models?search=whisper%20ggml&sort=downloads&direction=-1&limit=20"
        );
        assert_eq!(
            tree_url("ggerganov/whisper.cpp"),
            "https://huggingface.co/api/models/ggerganov/whisper.cpp/tree/main?recursive=false"
        );
        assert_eq!(
            resolve_url("ggerganov/whisper.cpp", "ggml-tiny.bin"),
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"
        );
    }

    #[test]
    fn parse_search_reads_id_model_id_and_downloads() {
        let json = r#"[
            {"id":"ggerganov/whisper.cpp","downloads":12345,"pipeline_tag":"automatic-speech-recognition"},
            {"modelId":"old/style/repo"},
            {"id":"no/downloads","downloads":null},
            {"id":17,"downloads":900},
            {"downloads":5}
        ]"#;
        assert_eq!(
            parse_search(json),
            vec![
                HfRepo {
                    id: "ggerganov/whisper.cpp".into(),
                    downloads: 12345
                },
                HfRepo {
                    id: "old/style/repo".into(),
                    downloads: 0
                },
                HfRepo {
                    id: "no/downloads".into(),
                    downloads: 0
                },
            ]
        );
    }

    #[test]
    fn parse_tree_keeps_only_bin_files_and_uses_lfs_size() {
        let json = r#"[
            {"type":"directory","path":"sub","size":0},
            {"type":"file","path":"README.md","size":12},
            {"type":"file","path":"ggml-tiny.bin","size":32000000},
            {"type":"file","path":"ggml-large.bin","size":0,"lfs":{"size":1100000000,"sha256":"abc"}},
            {"type":"file","path":"ggml-nolfs.bin"},
            {"path":"ggml-notyped.bin","size":77},
            "мусор"
        ]"#;
        assert_eq!(
            parse_tree(json),
            vec![
                HfFile {
                    path: "ggml-tiny.bin".into(),
                    size: 32_000_000
                },
                HfFile {
                    path: "ggml-large.bin".into(),
                    size: 1_100_000_000
                },
                HfFile {
                    path: "ggml-nolfs.bin".into(),
                    size: 0
                },
            ]
        );
    }

    #[test]
    fn broken_json_and_non_arrays_give_empty_lists() {
        for bad in ["{", "null", "{}", "\"text\""] {
            assert!(parse_search(bad).is_empty());
            assert!(parse_tree(bad).is_empty());
        }
    }

    #[test]
    fn filename_of_strips_query_and_rejects_paths_without_a_name() {
        assert_eq!(
            filename_of("https://huggingface.co/a/b/resolve/main/ggml-x.bin?download=true"),
            Some("ggml-x.bin".into())
        );
        assert_eq!(
            filename_of("https://host/dir/model.bin"),
            Some("model.bin".into())
        );
        assert_eq!(filename_of("https://host/dir/"), None);
        assert_eq!(filename_of("https://host/dir"), Some("dir".into()));
        assert_eq!(filename_of("example.com"), None);
        assert_eq!(filename_of(""), None);
    }
}
