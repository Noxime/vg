use crate::{async_trait, gfx::texture::Source, Size};

/// A typedef around `[f32; 4]` representing an RGBA color in sRGB colorspace
///
/// Note: Values outside of range `0.0` - `1.0` _may_ work, but are not
/// guaranteed to
pub type Color = [f32; 4];

/// Extension methods for color
pub trait ColorExt {
    /// Create a color from a hex triplet and an alpha of 100%
    fn hex(int: u32) -> Color;
}

impl ColorExt for Color {
    fn hex(int: u32) -> Color {
        let r = (int & 0xFF0000) >> 16;
        let g = (int & 0x00FF00) >> 8;
        let b = (int & 0x0000FF) >> 0;

        [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
    }
}

#[async_trait]
impl Source for Color {
    async fn load(&mut self) -> (Size, Vec<Color>) {
        ([1, 1], vec![*self])
    }
}
