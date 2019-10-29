#[derive(Debug)]
pub enum Error {
    Unk,
}

mod rend;
pub use self::rend::GRenderer;
mod tex;
pub use self::tex::GTexture;
mod surf;
pub use self::surf::GSurface;
