use crate::{Size, Color, Time};

mod raw;
mod lazy;
pub use raw::RawSource;
pub use lazy::LazySource;

/// Anything that can be used as a source for a texture
pub trait Source {
    fn load(&self) -> (Size, Vec<Color>);
    fn changed(&self) -> Time;
}
