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

/// Поиск/обзор моделей: запрос, фильтр задачи, сортировка и смещение для пагинации.
///
/// `sort` — одна из строк `downloads`, `likes`, `lastModified`.
/// Пустой `query` и выключенный `asr_only` означают «все модели подряд».
pub fn browse_url(query: &str, sort: &str, asr_only: bool, offset: usize) -> String {
    let mut url = String::from("https://huggingface.co/api/models?");
    if !query.is_empty() {
        url.push_str("search=");
        url.push_str(&url_encode(query));
        url.push('&');
    }
    if asr_only {
        url.push_str("filter=automatic-speech-recognition&");
    }
    url.push_str(&format!(
        "sort={sort}&direction=-1&limit=20&offset={offset}"
    ));
    url
}

/// Дерево репозитория: корень или конкретная подпапка на ветке `main`.
pub fn tree_url_at(repo: &str, subpath: &str) -> String {
    if subpath.is_empty() {
        format!("https://huggingface.co/api/models/{repo}/tree/main?recursive=false")
    } else {
        format!("https://huggingface.co/api/models/{repo}/tree/main/{subpath}")
    }
}

/// Прямая ссылка на файл репозитория для `curl -L`.
pub fn resolve_url(repo: &str, path: &str) -> String {
    format!("https://huggingface.co/{repo}/resolve/main/{path}")
}

/// Строка результата поиска: идентификатор репозитория, скачивания и лайки.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfRepo {
    pub id: String,
    pub downloads: u64,
    pub likes: u64,
}

/// Разбор ответа `/api/models?...`: битый JSON и не-массив дают пустой список.
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
            let likes = item.get("likes").and_then(|v| v.as_u64()).unwrap_or(0);
            Some(HfRepo {
                id: id.to_string(),
                downloads,
                likes,
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

/// Подпапки текущего уровня из ответа дерева (для навигации по репозиторию).
pub fn parse_tree_dirs(json: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return Vec::new();
    };
    let Some(items) = value.as_array() else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            if item.get("type").and_then(|v| v.as_str()) != Some("directory") {
                return None;
            }
            let path = item.get("path").and_then(|v| v.as_str())?;
            Some(path.to_string())
        })
        .collect()
}

/// Плоское имя файла для каталога моделей: последний сегмент пути с подпапками.
pub fn dest_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
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
    fn browse_url_combines_query_filter_sort_and_offset() {
        let base = "https://huggingface.co/api/models?";
        // Пустой запрос и без фильтра: только сортировка и пагинация.
        assert_eq!(
            browse_url("", "downloads", false, 0),
            format!("{base}sort=downloads&direction=-1&limit=20&offset=0")
        );
        // Запрос есть, фильтра нет.
        assert_eq!(
            browse_url("whisper ggml", "downloads", false, 20),
            format!("{base}search=whisper%20ggml&sort=downloads&direction=-1&limit=20&offset=20")
        );
        // Фильтр есть, запроса нет.
        assert_eq!(
            browse_url("", "likes", true, 0),
            format!("{base}filter=automatic-speech-recognition&sort=likes&direction=-1&limit=20&offset=0")
        );
        // Всё сразу, сортировка по обновлению и ненулевое смещение.
        assert_eq!(
            browse_url("wav2vec", "lastModified", true, 40),
            format!("{base}search=wav2vec&filter=automatic-speech-recognition&sort=lastModified&direction=-1&limit=20&offset=40")
        );
        // Пробелы в запросе кодируются, но сам запрос добавляется и при пробелах.
        assert_eq!(
            browse_url(" a/b ", "downloads", false, 0),
            format!("{base}search=%20a%2Fb%20&sort=downloads&direction=-1&limit=20&offset=0")
        );
    }

    #[test]
    fn tree_url_at_distinguishes_root_and_subpath() {
        assert_eq!(
            tree_url_at("ggerganov/whisper.cpp", ""),
            "https://huggingface.co/api/models/ggerganov/whisper.cpp/tree/main?recursive=false"
        );
        assert_eq!(
            tree_url_at("ggerganov/whisper.cpp", "sub/dir"),
            "https://huggingface.co/api/models/ggerganov/whisper.cpp/tree/main/sub/dir"
        );
    }

    #[test]
    fn api_urls_are_exact() {
        assert_eq!(
            resolve_url("ggerganov/whisper.cpp", "ggml-tiny.bin"),
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"
        );
    }

    #[test]
    fn parse_search_reads_id_downloads_and_likes() {
        let json = r#"[
            {"id":"ggerganov/whisper.cpp","downloads":12345,"likes":678,"pipeline_tag":"automatic-speech-recognition"},
            {"modelId":"old/style/repo","likes":null},
            {"id":"no/downloads","downloads":null,"likes":"12"},
            {"id":17,"downloads":900,"likes":5},
            {"downloads":5}
        ]"#;
        assert_eq!(
            parse_search(json),
            vec![
                HfRepo {
                    id: "ggerganov/whisper.cpp".into(),
                    downloads: 12345,
                    likes: 678,
                },
                HfRepo {
                    id: "old/style/repo".into(),
                    downloads: 0,
                    likes: 0,
                },
                HfRepo {
                    id: "no/downloads".into(),
                    downloads: 0,
                    likes: 0,
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
            {"type":"file","path":"sub/ggml-nested.bin","size":40000000},
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
                    path: "sub/ggml-nested.bin".into(),
                    size: 40_000_000
                },
                HfFile {
                    path: "ggml-nolfs.bin".into(),
                    size: 0
                },
            ]
        );
    }

    #[test]
    fn parse_tree_dirs_selects_only_directories() {
        let json = r#"[
            {"type":"directory","path":"sub","size":0},
            {"type":"file","path":"ggml-tiny.bin","size":32},
            {"type":"directory","path":"sub/deeper","size":0},
            {"type":"directory"},
            {"type":17,"path":"bad"},
            "мусор"
        ]"#;
        assert_eq!(parse_tree_dirs(json), vec!["sub", "sub/deeper"]);
        for bad in ["{", "null", "{}", "\"text\""] {
            assert!(parse_tree_dirs(bad).is_empty());
        }
    }

    #[test]
    fn broken_json_and_non_arrays_give_empty_lists() {
        for bad in ["{", "null", "{}", "\"text\""] {
            assert!(parse_search(bad).is_empty());
            assert!(parse_tree(bad).is_empty());
        }
    }

    #[test]
    fn dest_name_takes_the_last_path_segment() {
        assert_eq!(dest_name("sub/ggml.bin"), "ggml.bin");
        assert_eq!(dest_name("a/b/c/ggml.bin"), "ggml.bin");
        assert_eq!(dest_name("ggml.bin"), "ggml.bin");
        assert_eq!(dest_name(""), "");
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
