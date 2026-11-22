pub mod atspi;
pub mod audio;
#[cfg(unix)]
pub mod capture;
#[cfg(windows)]
pub mod capture_win;
pub mod combo;
pub mod config;
pub mod filter;
pub mod focus;
#[cfg(unix)]
pub mod hotkey;
#[cfg(windows)]
pub mod hotkey_win;
#[cfg(unix)]
pub mod inject;
#[cfg(windows)]
pub mod inject_win;
pub mod stt;

pub use audio::{load_wav, normalized_rms, resample_to_16k, silence_streak_ms, speech_level};
#[cfg(unix)]
pub use capture::Recorder;
#[cfg(windows)]
pub use capture_win::Recorder;
pub use combo::Combo;
pub use config::Config;
pub use stt::Engine;
