use crate::{Color, Size};

mod raw;
pub use raw::RawSource;

/// Anything that can be used as a source for a texture
#[crate::async_trait]
pub trait Source {
    /// Load the texture data from disk
    ///
    /// This function must be callable multiple times, since on some platforms
    /// (like mobile) the graphics context is lost on app switch and all
    /// resources must be reloaded
    async fn load(&mut self) -> (Size, Vec<Color>);
}
