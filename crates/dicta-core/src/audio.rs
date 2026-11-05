use anyhow::{bail, Context, Result};

/// Читает WAV (ожидаем 16 kHz mono s16 от pw-record) в i16-сэмплы.
///
/// Если канал больше одного — берём первый. Если частота не 16 kHz —
/// линейный ресэмпл (whisper всё равно внутри себя работает с 16 kHz).
pub fn load_wav(path: &std::path::Path) -> Result<(Vec<i16>, u32)> {
    let reader = hound::WavReader::open(path)
        .with_context(|| format!("не могу открыть {}", path.display()))?;
    let spec = reader.spec();

    if spec.channels == 0 {
        bail!("WAV без каналов: {}", path.display());
    }
    if spec.bits_per_sample != 16 && spec.bits_per_sample != 24 && spec.bits_per_sample != 32 {
        bail!(
            "неподдерживаемая глубина {} бит (нужно 16/24/32): {}",
            spec.bits_per_sample,
            path.display()
        );
    }

    let channels = spec.channels as usize;
    let flat: Vec<i16> = match spec.bits_per_sample {
        16 => reader
            .into_samples::<i16>()
            .map(|s| s.map_err(|e| anyhow::anyhow!("битый WAV: {e}")))
            .collect::<Result<Vec<_>, _>>()?,
        // 24/32-битные данные сужаем до 16, усредняя каналы сразу нельзя: сначала нормализуем
        bits => {
            let normalized: Vec<f32> = reader
                .into_samples::<i32>()
                .map(|s| -> anyhow::Result<f32> {
                    let s = s.map_err(|e| anyhow::anyhow!("битый WAV: {e}"))?;
                    Ok((s as f32) / ((1i64 << (bits - 1)) as f32))
                })
                .collect::<anyhow::Result<Vec<f32>>>()?;
            let mut out = Vec::with_capacity(normalized.len() / channels);
            for frame in normalized.chunks(channels) {
                out.push((frame[0] * 32767.0).clamp(-32768.0, 32767.0) as i16);
            }
            out
        }
    };

    let frames = deinterleave(flat, channels);
    Ok((frames, spec.sample_rate))
}

fn deinterleave(flat: Vec<i16>, channels: usize) -> Vec<i16> {
    if channels <= 1 {
        return flat;
    }
    flat.chunks(channels).map(|c| c[0]).collect()
}

/// Линейный ресэмпл в 16 kHz, если источник в другой частоте.
pub fn resample_to_16k(samples: Vec<i16>, rate: u32) -> Vec<i16> {
    if rate == 16000 || samples.is_empty() {
        return samples;
    }
    let ratio = 16000.0 / rate as f64;
    let out_len = ((samples.len() as f64) * ratio).round().max(1.0) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src = i as f64 / ratio;
        let idx = src.floor() as usize;
        let frac = src - idx as f64;
        let a = samples[idx.min(samples.len() - 1)] as f64;
        let b = samples[(idx + 1).min(samples.len() - 1)] as f64;
        out.push((a + (b - a) * frac).clamp(-32768.0, 32767.0) as i16);
    }
    out
}

/// i16 -> f32 в диапазоне [-1, 1], как ожидает whisper_full.
pub fn to_f32(samples: &[i16]) -> Vec<f32> {
    samples.iter().map(|s| *s as f32 / 32768.0).collect()
}

/// RMS последних `bytes` байт i16-потока, нормализованный в 0..1
/// (для индикатора уровня в UI; порог как в opencode-плагине: -55..-15 dB).
pub fn normalized_rms(samples: &[i16]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f64 = samples
        .iter()
        .map(|s| {
            let v = *s as f64 / 32768.0;
            v * v
        })
        .sum();
    let db = (20.0 * (sum / samples.len() as f64).max(1e-9).sqrt().log10()) as f32;
    ((db + 55.0) / 40.0).clamp(0.0, 1.0)
}

/// Уровень речи: максимум индикатора по окнам 300 мс.
/// Речь — это всплески, поэтому средний RMS по всей записи недооценивает короткую
/// и тихую диктовку и переоценивает ровный фоновый шум. Замеры на этой машине:
/// фон в комнате 0.34-0.35, русская речь 0.9-1.0 (по среднему: 0.45..0.95).
pub fn speech_level(samples: &[i16]) -> f32 {
    const WIN: usize = 4800; // 300 мс при 16 кГц
    const STEP: usize = 1600; // 100 мс
    if samples.is_empty() {
        return 0.0;
    }
    if samples.len() <= WIN {
        return normalized_rms(samples);
    }
    let mut best = 0.0f32;
    let mut i = 0;
    while i < samples.len() {
        let end = (i + WIN).min(samples.len());
        best = best.max(normalized_rms(&samples[i..end]));
        i += STEP;
    }
    best
}

/// Сколько миллисекунд подряд последние замеры уровня держатся ниже `threshold`.
///
/// `levels` — уровни от новых к старым (как их собирает watchdog демона), `tick_ms` —
/// интервал между замерами. Используется авто-остановкой записи по тишине.
pub fn silence_streak_ms(levels: &[f32], threshold: f32, tick_ms: u64) -> u64 {
    let mut n = 0usize;
    for l in levels {
        if *l < threshold {
            n += 1;
        } else {
            break;
        }
    }
    (n as u64) * tick_ms
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noise(amp: i16, n: usize) -> Vec<i16> {
        (0..n)
            .map(|i| if i % 2 == 0 { amp } else { -amp })
            .collect()
    }

    #[test]
    fn speech_level_uses_loudest_window() {
        // 2 с фона + 0.3 с «речи» в середине
        let mut samples = noise(100, 16000 * 2);
        samples.splice(16000..16000 + 4800, noise(8000, 4800));
        let level = speech_level(&samples);
        assert!(level > 0.7, "всплеск не замечен: {level}");
        assert!(
            normalized_rms(&samples) < level,
            "среднее не должно быть строже пика"
        );
    }

    #[test]
    fn silence_streak_counts_only_the_tail() {
        // уровни от новых к старым: 3 тихих замера по 100 мс
        assert_eq!(
            silence_streak_ms(&[0.10, 0.12, 0.09, 0.80, 0.90], 0.22, 100),
            300
        );
        // свежий громкий замер обнуляет очередь тишины
        assert_eq!(silence_streak_ms(&[0.80, 0.10, 0.10], 0.22, 100), 0);
        assert_eq!(silence_streak_ms(&[], 0.22, 100), 0);
    }

    #[test]
    fn speech_level_rejects_flat_noise() {
        let level = speech_level(&noise(100, 16000 * 3));
        assert!(level < 0.2, "ровный шум прошёл гейтер: {level}");
    }
}
