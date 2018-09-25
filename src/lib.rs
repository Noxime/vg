#[macro_use] extern crate log;
extern crate pretty_env_logger;

pub mod graphics;
pub mod vectors;

/// initialize static parts of kea.
/// This should be the first function you call
pub fn init() {
    pretty_env_logger::init();

    info!("Launching Kea");
}