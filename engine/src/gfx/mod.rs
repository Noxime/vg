//! Graphics

use crate::{Color, Matrix, Size};

pub mod texture;
use texture::{Source, Texture};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Backend {
    name: String,
    available: bool,
    active: bool,
}

impl Backend {
    /// Human readable name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the backend is available on this particular computer
    ///
    /// Not all backends will be available. For example, your game might
    /// be built with support for both OpenGL and Vulkan, but the user only has
    /// graphics drivers for OpenGL. Make sure to check this before calling
    /// [`Gfx::change_backend`]
    pub fn available(&self) -> bool {
        self.available
    }

    /// Is this the currently active backend
    ///
    /// Note: This does not update upon changing the backend. If you do change,
    /// make sure to refresh your copy of the backends by calling
    /// [`Gfx::backends`] again
    pub fn active(&self) -> bool {
        self.active
    }
}

pub struct Gfx {
    gfx: Box<dyn GfxTrait>,
}

impl Gfx {
    #[cfg_attr(not(feature = "dev-docs"), doc(hidden))]
    pub fn new(gfx: Box<dyn GfxTrait>) -> Self {
        Self {
            gfx
        }
    }
    /// Change the vsync setting
    /// 
    /// Note: This is best-effort, as vsync settings can often be overwritten
    /// from the graphics driver settings.
    /// 
    /// You should also keep in mind that vsync does not always mean 60fps, so
    /// you should not lock your game step to the assumption of one frame being
    /// ~16ms
    pub fn vsync(&mut self, vsync: bool) {
        self.gfx.vsync(vsync)
    }

    /// Get all the graphics APIs this platform supports
    pub fn backends(&self) -> Vec<Backend> {
        self.gfx.backends()
    }

    /// Change to a backend
    ///
    /// Note: This may be quite slow, as resources may have to be reloaded
    pub async fn change_backend(&mut self, backend: usize) -> Result<(), ()> {
        self.gfx.change_backend(backend).await
    }

    /// Create a new texture from a texture [`Source`]
    pub async fn texture(&mut self, source: impl Source + 'static) -> Texture {
        Texture::new(self.gfx.texture(Box::new(source)).await)
    }

    /// Present all the draw operations to the screen, and possibly wait for
    /// vsync
    pub async fn present(&mut self) {
        self.gfx.present().await
    }
}

#[cfg_attr(not(feature = "dev-docs"), doc(hidden))]
#[crate::async_trait(?Send)]
pub trait GfxTrait: Target {
    fn backends(&self) -> Vec<Backend>;
    async fn change_backend(&mut self, backend: usize) -> Result<(), ()>;

    /// Set vsync state
    fn vsync(&mut self, vsync: bool);

    /// Upload the texture to GPU
    async fn texture(&mut self, source: Box<dyn Source>) -> Box<dyn texture::TextureTrait>;

    /// Return a future that, when awaited, will present the current frame to
    /// the display and wait for vblank if necessary
    async fn present(&mut self);
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

impl Target for Gfx {
    fn size(&self) -> Size {
        self.gfx.size()
    }

    fn fill(&mut self, color: Color) {
        self.gfx.fill(color)
    }

    fn draw(&mut self, texture: &Texture, matrices: &[Matrix]) {
        self.gfx.draw(texture, matrices)
    }
}