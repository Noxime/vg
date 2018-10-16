
#[derive(Debug)]
pub enum GraphicsError {
    NoAdapter,
    NoWindowBuilder,
    WindowError,
}