use crate::{Color, Size};

mod raw;
pub use raw::RawSource;

/// Anything that can be used as a source for a texture
#[crate::async_trait]
pub trait Source {
    /// Load the texture data from disk
    async fn load(&mut self) -> (Size, Vec<Color>);

    /// Check if the data has changed, and load it if it has
    async fn changed(&mut self) -> Option<(Size, Vec<Color>)>;
}
