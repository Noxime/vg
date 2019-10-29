//! # Overview
//! The renderer abstraction in vg is very basic, and really is only useful for
//! basic sprite rendering. This is on purpose, as sprite games are only ones
//! I plan on making, but if you have good ideas on how to extend the api, drop
//! me a PR ;)
//!
//! There are 4 traits that provide a generic way of rendering sprites (called
//! textures) in vg:
//!
//! ## `Renderer`
//! This your instance of a rendering backend, and you use it to get access to
//! your window (so you can draw to it)
//! ## `Surface`
//! Surface represents a window that you can draw to. It has 2 methods:
//! * [`capture`](renderer::Surface::capture) screenshots the window and returns it as
//! texture
//! * [`present`](renderer::Surface::present) ends the current frame and swaps the buffer
//! to the window, possibly waiting until next Vblank
//! ## `Texture`
//! Texture is what it sounds like, an RGBA texture in GPU memory that you can
//! [`draw`](renderer::Target::draw) to any valid [`Target`](renderer::Target)
//! ## `Target`
//! Target is anything you can render [`Texture`](renderer::Texture)s into. A [`Surface`](renderer::Surface) is a
//! target, but so are [`Texture`](renderer::Texture)s
//!

pub mod font;

pub type Color = [f32; 4];
pub type Size = [usize; 2];

#[derive(Debug, Clone)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
}

impl Transform {
    pub fn translate(&self, x: f32, y: f32) -> Transform {
        Transform {
            x: x + self.x,
            y: y + self.y,
            ..*self
        }
    }

    pub fn rotate(&self, radians: f32) -> Transform {
        Transform {
            rotation: self.rotation + radians,
            ..*self
        }
    }

    pub fn scale(&self, x: f32, y: f32) -> Transform {
        Transform {
            scale_x: self.scale_x * x,
            scale_y: self.scale_y * y,
            ..*self
        }
    }
}

impl Default for Transform {
    fn default() -> Transform {
        Transform {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct View {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale: Scale,
    pub pixels_per_unit: f32,
}

impl View {
    pub fn dimensions(&self, size: &Size) -> [f32; 2] {
        match self.scale {
            Scale::Vertical(v) => [v * size[0] as f32 / size[1] as f32, v],
            Scale::Horizontal(v) => [v, v * size[1] as f32 / size[0] as f32],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Scale {
    Vertical(f32),
    Horizontal(f32),
}

/// Various shading options when rendering
#[derive(Debug)]
pub struct Shading {
    pub add: Color,
    pub multiply: Color,
}

impl Default for Shading {
    fn default() -> Shading {
        Shading {
            add: [0.0; 4],
            multiply: [1.0; 4],
        }
    }
}

/// Instance of a rendering backend
///
/// See the [module documentation](crate::renderer) on how to use the rendering api
pub trait Renderer: Sized {
    /// A user friendly name of our rendering engine
    const NAME: &'static str;

    /// A texture that you can draw into a target
    type Texture: Texture<Self>;
    /// The window on whatever platform you are
    type Surface: Surface<Self>;
    /// Get the active surface (window)
    fn surface(&mut self) -> &mut Self::Surface;
}

pub trait Texture<R: Renderer>: Target<R> {
    /// Create a new texture from size and with given color
    fn new(renderer: &mut R, size: &Size, color: &Color) -> Self;
    /// Create a new texture from size and the given data iterator. **The iterator
    /// must return at least width * height items**
    fn from_data(renderer: &mut R, size: &Size, data: &Vec<Color>) -> Self;
    /// Clone the texture into a new object
    fn clone(&self) -> Self;
}

pub trait Surface<R: Renderer>: Target<R> {
    /// Capture the window contents and return them as a new Texture
    fn capture(&self) -> R::Texture;
    /// Flush the rendering queue and present the final image to the display,
    /// possibly waiting for next vertical blank
    fn present(&mut self, vsync: bool) -> Box<dyn std::future::Future<Output=()> + Unpin>;
}

/// Common trait for both Texture and Surface, where you can draw to
pub trait Target<R: Renderer> {
    /// Get the size of this target in pixels
    fn size(&self) -> Size;
    /// Set (clear) the target to some specific color
    fn set(&mut self, color: &Color);
    /// Draw a texture into this target, by transforming it with the provided
    /// transform.
    fn draw(&mut self, texture: &R::Texture, shading: &Shading, view: &View, transform: &Transform);
}
