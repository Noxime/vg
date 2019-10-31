use crate::{gfx::texture::Source, Color, Size, Time, EPOCH};

/// A source that will first resolve to `initial`, and later to `end` when it
/// becomes available
pub struct LazySource<T: Source, U: Source> {
    initial: T,
    end: U,
}

impl<T: Source, U: Source> LazySource<T, U> {
    /// Create a new source from raw size and data
    pub fn new(initial: T, end: U) -> LazySource<T, U> {
        LazySource {
            initial,
            end
        }
    }
}

impl<T: Source, U: Source> Source for LazySource<T, U> {
    fn load(&self) -> (Size, Vec<Color>) {
        unimplemented!()
    }

    fn changed(&self) -> Time {
        EPOCH
    }
}