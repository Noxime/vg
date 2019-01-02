pub type Color = [f32; 4];
pub type Coordinate = [isize; 2];
pub type Size = [usize; 2];

pub trait Renderer: Sized {
    /// A user friendly name of our rendering engine
    const NAME: &'static str;

    /// A texture that you can draw into a target
    type Texture: Texture<Self>;
    /// The window on whatever platform you are
    type Surface: Surface<Self>;
    /// Get the active window
    fn surface(&mut self) -> &mut Self::Surface;
}

pub trait Texture<R: Renderer>: Target<R> {
    /// Create a new texture from size and with given color
    fn new(size: &Size, color: &Color) -> Self;
    /// Clone the texture into a new object
    fn clone(&self) -> Self;
    /// Scale the texture to the new dimensions
    fn scale(&mut self, size: &Size);
}

pub trait Surface<R: Renderer>: Target<R> {
    /// Capture the window contents and return them as a new Texture
    fn capture(&self) -> R::Texture;
}

/// Common trait for both Texture and Surface, where you can draw to
pub trait Target<R: Renderer> {
    /// Get the size of this target in pixels
    fn size(&self) -> Size;
    /// Set (clear) the target to some specific color
    fn set(&mut self, color: &Color);
    /// Draw a texture into this target, (x, y) == (0, 0) being top left corner
    /// of the target and texture anchor point being top left corner
    fn draw(&mut self, texture: R::Texture, coords: Coordinate);
}