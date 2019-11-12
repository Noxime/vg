use crate::{gfx::texture::Source, Color, Size};

/// Raw, in memory texture source
pub struct RawSource(Size, Vec<Color>);

impl RawSource {
    /// Create a new source from raw size and data
    pub fn new(size: Size, data: Vec<Color>) -> RawSource {
        RawSource(size, data)
    }
}

#[crate::async_trait]
impl Source for RawSource {
    async fn load(&self) -> (Size, Vec<Color>) {
        (self.0, self.1.clone())
    }

    async fn changed(&self) -> Option<(Size, Vec<Color>)> {
        None
    }
}

impl From<(Size, Vec<Color>)> for RawSource {
    fn from(from: (Size, Vec<Color>)) -> RawSource {
        RawSource::new(from.0, from.1)
    }
}