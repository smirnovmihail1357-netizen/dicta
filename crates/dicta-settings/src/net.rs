//! Клиент сокета демона: одна команда — одна строка ответа.
//!
//! Валидацию значений здесь не делаем: демон сам проверит key/value, применит
//! изменение к живому конфигу и запишет config.toml. Окно только показывает его ответ.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

/// Одно постоянное соединение: writer/reader — два дескриптора одного сокета.
pub struct Client {
    writer: UnixStream,
    reader: BufReader<UnixStream>,
}

impl Client {
    /// Подключиться к сокету из конфига (тот же путь, что использует dictactl).
    pub fn connect(socket: &Path) -> anyhow::Result<Self> {
        let stream = UnixStream::connect(socket).map_err(|err| {
            anyhow::anyhow!(
                "демон не отвечает на {} ({err}) — запусти systemctl --user start dictad.service",
                socket.display()
            )
        })?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(30)))?;
        let writer = stream.try_clone()?;
        let reader = BufReader::new(stream);
        Ok(Self { writer, reader })
    }

    /// Отправить готовую команду (с завершающим `\n`) и прочитать один ответ целиком.
    pub fn call(&mut self, line: &str) -> anyhow::Result<serde_json::Value> {
        self.writer.write_all(line.as_bytes())?;
        self.writer.flush()?;
        let mut response = String::new();
        if self.reader.read_line(&mut response)? == 0 {
            anyhow::bail!("демон закрыл соединение");
        }
        Ok(serde_json::from_str(response.trim())?)
    }
}

/// `{"cmd":"settings"}` — снимок настроек и список моделей.
pub fn settings_json() -> &'static str {
    "{\"cmd\":\"settings\"}\n"
}

/// `{"cmd":"set","key":...,"value":...}` — значение всегда строкой (так ждёт демон).
pub fn set_json(key: &str, value: &str) -> String {
    // key приходит из нашего кода (фиксированные строки), экранируем только value.
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("{{\"cmd\":\"set\",\"key\":\"{key}\",\"value\":\"{escaped}\"}}\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_request_has_no_arguments() {
        assert_eq!(settings_json(), "{\"cmd\":\"settings\"}\n");
    }

    #[test]
    fn set_json_escapes_quotes_and_backslashes() {
        assert_eq!(
            set_json("model", "C:\\models\\ggml-\"x\".bin"),
            "{\"cmd\":\"set\",\"key\":\"model\",\"value\":\"C:\\\\models\\\\ggml-\\\"x\\\".bin\"}\n"
        );
    }
}
