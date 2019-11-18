//! Graphics

use crate::{Color, Matrix, Size};

pub mod texture;
use texture::{Source, Texture};

#[crate::async_trait]
pub trait Gfx: Target {
    /// Get the currently active backend as a readable string
    fn api(&self) -> String;
    /// Create a new texture from a texture [`Source`]
    fn texture(&self, source: Box<dyn Source>) -> Texture;
    /// Present all the draw operations to the screen, and possibly wait for
    /// vsync
    fn present(&mut self) -> Present;
}

pub trait Target {
    /// Get the size of this target area in pixels
    fn size(&self) -> Size;
    /// Fill the target with a solid color
    fn fill(&mut self, color: Color);
    /// Draw a texture instanced by the transform matrices provided
    ///
    /// # Example
    ///
    /// The following code will draw `ferris` filling the whole `Target`
    /// ```rust
    /// vg.draw(&ferris, &[Matrix::IDENTITY]);
    /// ```
    fn draw(&mut self, texture: &Texture, matrices: &[Matrix]);
}

pub struct Present(#[doc(hidden)] pub Box<dyn std::future::Future<Output = ()>>);
