use crate::Vg;

impl Vg {
    /// Get seconds since game start
    pub fn time(&self) -> f64 {
        self.runtime
    }

    /// Seconds used for _last_ frame
    pub fn delta_time(&self) -> f64 {
        self.delta
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn now() -> f64 {
    std::time::SystemTime::UNIX_EPOCH
        .elapsed()
        .expect("System clock difted backwards")
        .as_secs_f64()
}
#[cfg(target_arch = "wasm32")]
pub(crate) fn now() -> f64 {
    stdweb::web::Date::now() / 1000.0
}
