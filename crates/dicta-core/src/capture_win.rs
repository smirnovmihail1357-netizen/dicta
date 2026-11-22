//! WASAPI-захват через cpal. Контракт паритетен `capture::Recorder`:
//! `start` → `stop` → Vec<i16> 16 кГц, `elapsed`, `level`.
//!
//! Native default_input_config (НЕ принудительные 16 кГц — Pitfall 1:
//! AUTOCONVERTPCM даёт тишину на Win11 24H2); децимация — существующим
//! `audio::resample_to_16k`. Сэмплы живут ТОЛЬКО в памяти.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use crate::config::Config;

/// ~120 с сэмплов при частоте WASAPI — страховка от утечки памяти в callback.
const MAX_SECONDS: usize = 120;

fn pcm_f32_to_i16(sample: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * 32767.0) as i16
}

fn is_overflow(len: usize, max: usize) -> bool {
    len > max
}

pub struct Recorder {
    stream: cpal::Stream,
    samples: Arc<Mutex<Vec<i16>>>,
    rate: u32,
    started: Instant,
    /// Callback продолжает приходить после потолка — копим флаг, не данные.
    overflow: Arc<AtomicBool>,
}

impl Recorder {
    pub fn start(cfg: &Config) -> Result<Self> {
        let host = cpal::default_host();
        let device = match &cfg.mic {
            Some(mic) => host
                .devices()
                .context("не удалось перечислить устройства WASAPI")?
                .find(|d| d.name().map(|n| n.contains(mic.as_str())).unwrap_or(false))
                .ok_or_else(|| anyhow::anyhow!("микрофон '{mic}' не найден (WASAPI)"))?,
            None => host
                .default_input_device()
                .ok_or_else(|| anyhow::anyhow!("нет устройства ввода (WASAPI)"))?,
        };

        // Native rate/channels: просить 16 кГц у WASAPI нельзя (Pitfall 1).
        let config = device
            .default_input_config()
            .context("нет конфигурации ввода WASAPI")?;
        let rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        let sample_format = config.sample_format();
        let stream_config: cpal::StreamConfig = config.into();

        let samples = Arc::new(Mutex::new(Vec::<i16>::with_capacity(16_000 * 30)));
        let overflow = Arc::new(AtomicBool::new(false));
        let max_samples = (rate as usize) * MAX_SECONDS;
        let err_fn = |err| eprintln!("dictad: cpal: {err}");

        let stream = match sample_format {
            cpal::SampleFormat::I16 => {
                let buf = samples.clone();
                let ovf = overflow.clone();
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _| {
                        let mut out = buf.lock().unwrap();
                        if is_overflow(out.len(), max_samples) {
                            ovf.store(true, Ordering::Relaxed);
                            return;
                        }
                        for frame in data.chunks_exact(channels) {
                            out.push(frame[0]); // первый канал (паритет deinterleave)
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            cpal::SampleFormat::F32 => {
                let buf = samples.clone();
                let ovf = overflow.clone();
                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _| {
                        let mut out = buf.lock().unwrap();
                        if is_overflow(out.len(), max_samples) {
                            ovf.store(true, Ordering::Relaxed);
                            return;
                        }
                        for frame in data.chunks_exact(channels) {
                            out.push(pcm_f32_to_i16(frame[0]));
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            other => anyhow::bail!("cpal: неподдерживаемый формат {other:?}"),
        };
        stream.play().context("не удалось запустить cpal-поток")?;

        Ok(Self {
            stream,
            samples,
            rate,
            started: Instant::now(),
            overflow,
        })
    }

    /// Остановить поток и вернуть сэмплы, приведённые к 16 кГц.
    pub fn stop(mut self) -> Result<Vec<i16>> {
        self.stream.pause().context("не удалось остановить cpal-поток")?;
        let raw = std::mem::take(&mut *self.samples.lock().unwrap());
        if self.overflow.load(Ordering::Relaxed) {
            eprintln!("dictad: cpal: буфер переполнен, запись обрезана");
        }
        Ok(crate::audio::resample_to_16k(raw, self.rate))
    }

    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    /// Уровень по хвосту ~40 мс буфера в памяти (паритет capture.rs:79-93).
    pub fn level(&self) -> f32 {
        let guard = self.samples.lock().unwrap();
        let window = ((self.rate as usize / 25).max(64)).min(guard.len());
        crate::audio::normalized_rms(&guard[guard.len() - window..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_samples_are_clamped_to_i16_range() {
        assert_eq!(pcm_f32_to_i16(0.0), 0);
        assert_eq!(pcm_f32_to_i16(1.0), 32767);
        assert_eq!(pcm_f32_to_i16(-1.0), -32767);
        assert_eq!(pcm_f32_to_i16(2.5), 32767);
        assert_eq!(pcm_f32_to_i16(-2.5), -32767);
        assert_eq!(pcm_f32_to_i16(0.5), 16383);
    }

    #[test]
    fn overflow_triggers_only_above_ceiling() {
        let max = 16_000 * 120;
        assert!(!is_overflow(max, max));
        assert!(is_overflow(max + 1, max));
        assert!(!is_overflow(0, max));
    }
}
