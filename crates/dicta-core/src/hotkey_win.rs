//! Глобальная горячая клавиша Windows (заглушка Wave 1).
//!
//! Тело появится в Task 4 (07-01): RegisterHotKey с hWnd=NULL → очередь потока,
//! без создания окна. Сигнатура уже паритетна `hotkey::listen`, поэтому общий
//! блок DICTA_HOTKEY/'off'/backoff в dictad не меняется.

/// Честная заглушка: горячую клавишу пока зарегистрировать нельзя.
pub fn listen(_spec: &str, _on_press: impl FnMut()) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "горячая клавиша на Windows появится в задаче 4 (07-01)",
    ))
}
