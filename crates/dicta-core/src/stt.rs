use std::ffi::{c_char, c_void};

use anyhow::{Context, Result};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use whisper_rs_sys::{
    ggml_log_level, ggml_log_level_GGML_LOG_LEVEL_WARN, ggml_log_set, whisper_log_set,
};

use crate::audio::to_f32;
use crate::config::Config;
use crate::filter::clean;

/// Шум whisper.cpp ("decoder 1: score = ...", "seek = ...") глушим: в демоне он только мешает.
/// В stderr остаётся warn и error.
static QUIET_LOG_INSTALLED: std::sync::Once = std::sync::Once::new();

unsafe extern "C" fn quiet_log(level: ggml_log_level, text: *const c_char, _: *mut c_void) {
    if level < ggml_log_level_GGML_LOG_LEVEL_WARN || text.is_null() {
        return;
    }
    let msg = unsafe { std::ffi::CStr::from_ptr(text) }.to_string_lossy();
    eprint!("dicta/whisper: {msg}");
}

fn install_quiet_logging() {
    QUIET_LOG_INSTALLED.call_once(|| unsafe {
        whisper_log_set(Some(quiet_log), std::ptr::null_mut());
        ggml_log_set(Some(quiet_log), std::ptr::null_mut());
    });
}

/// Один экземпляр модели внутри процесса демона.
///
/// VRAM-бюджет: модель держится в памяти постоянно, выгружается только при
/// завершении демона. Поэтому конфигурация по умолчанию использует ровно одну
/// модель и не грузит вторую «про запас».
pub struct Engine {
    ctx: WhisperContext,
    cfg: Config,
}

impl Engine {
    pub fn load(cfg: &Config) -> Result<Self> {
        install_quiet_logging();
        if !cfg.model.exists() {
            anyhow::bail!(
                "модель не найдена: {} (проверь model в ~/.config/dicta/config.toml)",
                cfg.model.display()
            );
        }

        let mut params = WhisperContextParameters::default();
        params.use_gpu = cfg.use_gpu;
        params.flash_attn = cfg.flash_attn;
        params.gpu_device = cfg.gpu_device;

        let ctx = WhisperContext::new_with_params(&cfg.model, params)
            .with_context(|| format!("whisper не смог загрузить {}", cfg.model.display()))?;

        Ok(Self { ctx, cfg: cfg.clone() })
    }

    /// i16-сэмплы 16 kHz -> распознаванный текст.
    pub fn transcribe(&self, samples: &[i16]) -> Result<String> {
        if samples.is_empty() {
            return Ok(String::new());
        }

        let strategy = if self.cfg.beam_size <= 1 {
            SamplingStrategy::Greedy { best_of: 1 }
        } else {
            SamplingStrategy::BeamSearch {
                beam_size: self.cfg.beam_size,
                patience: -1.0,
            }
        };

        let mut params = FullParams::new(strategy);
        params.set_language(Some(self.cfg.language.as_str()));
        params.set_n_threads(self.cfg.n_threads);
        params.set_initial_prompt(&self.cfg.effective_prompt());
        params.set_suppress_blank(true);
        params.set_suppress_nst(true);
        params.set_no_timestamps(true);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        let audio = to_f32(samples);
        let mut state = self
            .ctx
            .create_state()
            .context("не создал whisper-состояние")?;
        state
            .full(params, &audio)
            .context("whisper_full вернул ошибку")?;

        let mut out = String::new();
        for segment in state.as_iter() {
            out.push_str(&segment.to_str_lossy()?);
            out.push('\n');
        }

        Ok(clean(&out))
    }

    /// Модель мультиязычная? (en-модели распознают только английский)
    pub fn is_multilingual(&self) -> bool {
        self.ctx.is_multilingual()
    }
}
