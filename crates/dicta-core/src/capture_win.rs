//! WASAPI-захват через cpal (заглушка Wave 1).
//!
//! Реальный Recorder появится в Task 3 (07-01) с тем же контрактом, что у
//! `capture::Recorder`: start → stop → Vec<i16> 16 кГц, elapsed, level.
//! dictad не заметит подмены модуля.

/// Заглушка с API-контрактом Linux-Recorder'а.
pub struct Recorder;

impl Recorder {
    /// Честная заглушка: устройство ввода пока не читается.
    pub fn start(_cfg: &crate::config::Config) -> anyhow::Result<Self> {
        anyhow::bail!("захват звука на Windows появится в задаче 3 (07-01)")
    }

    /// Честная заглушка: сэмплов ещё нет.
    pub fn stop(self) -> anyhow::Result<Vec<i16>> {
        anyhow::bail!("захват звука на Windows появится в задаче 3 (07-01)")
    }

    pub fn elapsed(&self) -> std::time::Duration {
        std::time::Duration::ZERO
    }

    pub fn level(&self) -> f32 {
        0.0
    }
}
