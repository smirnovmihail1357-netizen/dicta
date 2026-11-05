use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use crate::audio::load_wav;
use crate::config::Config;

/// Запись через PipeWire: `pw-record` пишет 16 kHz mono s16 WAV в temp-файл.
pub struct Recorder {
    child: Child,
    file: PathBuf,
    started: Instant,
}

impl Recorder {
    pub fn start(cfg: &Config) -> Result<Self> {
        let file = std::env::temp_dir().join(format!(
            "dicta-{}-{}.wav",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis()
        ));

        let mut cmd = Command::new("pw-record");
        if let Some(mic) = &cfg.mic {
            cmd.args(["--target", mic]);
        }
        cmd.args(["--rate", "16000", "--channels", "1", "--format", "s16"]);
        cmd.arg(&file);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        let child = cmd
            .spawn()
            .context("не удалось запустить pw-record (поставлен pipewire?)")?;

        Ok(Self {
            child,
            file,
            started: Instant::now(),
        })
    }

    /// Останавливает запись (SIGINT, как в opencode-плагине) и возвращает сэмплы.
    pub fn stop(mut self) -> Result<Vec<i16>> {
        let pid = self.child.id() as i32;
        // SIGINT, а не SIGTERM: pw-record успевает дописать заголовок WAV.
        unsafe { libc::kill(pid, libc::SIGINT) };

        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match self.child.try_wait()? {
                Some(_) => break,
                None if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
                None => {
                    let _ = self.child.kill();
                    let _ = self.child.wait()?;
                    break;
                }
            }
        }

        let path = self.file.clone();
        let (samples, rate) = load_wav(&path)
            .with_context(|| format!("запись не создана или битая: {}", path.display()))?;
        Ok(crate::audio::resample_to_16k(samples, rate))
    }

    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    /// Текущий уровень 0..1 по недописанному WAV (для волны в UI).
    pub fn level(&self) -> f32 {
        let Ok(bytes) = std::fs::read(&self.file) else {
            return 0.0;
        };
        // WAV-заголовок 44 байта; берём последние ~40 мс (1280 байт s16).
        if bytes.len() < 44 + 1280 {
            return 0.0;
        }
        let tail = &bytes[bytes.len() - 1280..];
        let samples: Vec<i16> = tail
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]))
            .collect();
        crate::audio::normalized_rms(&samples)
    }

    pub fn file(&self) -> &PathBuf {
        &self.file
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.file);
    }
}

/// Записать `secs` секунд и вернуть сэмпллы (для калибровки уровня).
pub fn record_samples(cfg: &Config, secs: u64) -> Result<Vec<i16>> {
    let rec = Recorder::start(cfg)?;
    std::thread::sleep(Duration::from_secs(secs));
    rec.stop()
}

/// Для отладки: записать `secs` секунд и сохранить в файл.
pub fn record_to(cfg: &Config, secs: u64, out: &std::path::Path) -> Result<()> {
    let rec = Recorder::start(cfg)?;
    std::thread::sleep(Duration::from_secs(secs));
    let samples = rec.stop()?;
    let mut f = std::fs::File::create(out)?;
    f.write_all(&wav_header(samples.len()))?;
    for s in &samples {
        f.write_all(&s.to_le_bytes())?;
    }
    Ok(())
}

fn wav_header(data_len: usize) -> [u8; 44] {
    let data_size = (data_len * 2) as u32;
    let total = (44 + data_len * 2) as u32;
    let mut h = [0u8; 44];
    h[0..4].copy_from_slice(b"RIFF");
    h[4..8].copy_from_slice(&(total - 8).to_le_bytes());
    h[8..12].copy_from_slice(b"WAVE");
    h[12..16].copy_from_slice(b"fmt ");
    h[16..20].copy_from_slice(&16u32.to_le_bytes());
    h[20..22].copy_from_slice(&1u16.to_le_bytes()); // PCM
    h[22..24].copy_from_slice(&1u16.to_le_bytes()); // mono
    h[24..28].copy_from_slice(&16000u32.to_le_bytes());
    h[28..32].copy_from_slice(&32000u32.to_le_bytes()); // byte rate
    h[32..34].copy_from_slice(&2u16.to_le_bytes()); // block align
    h[34..36].copy_from_slice(&16u16.to_le_bytes());
    h[36..40].copy_from_slice(b"data");
    h[40..44].copy_from_slice(&data_size.to_le_bytes());
    h
}
