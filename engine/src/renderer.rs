pub type Color = [f32; 4];
pub type Size = [usize; 2];
/// A matrix representing the transform of a texture
#[derive(Clone, Copy, Debug)]
pub struct Matrix([[f32; 3]; 3]);

impl Matrix {
    /// Create an identity matrix
    /// 
    /// ```
    /// [1, 0, 0]
    /// [0, 1, 0]
    /// [0, 0, 1]
    /// ```
    pub fn identity() -> Self {
        Matrix([
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ])
    }

    pub fn from(raw: &[[f32; 3]; 3]) -> Self {
        Matrix(*raw)
    }

    /// Scale this matrix around the origin
    pub fn scale(&mut self, x: f32, y: f32) {
        self.0[0][0] *= x;
        self.0[1][1] *= y;

        // ..?
        // self.0[0][2] *= x;
        // self.0[1][2] *= x;
    }

    /// Return a scaled clone of this matrix
    pub fn scaled(&self, x: f32, y: f32) -> Matrix {
        let mut m = self.clone();
        m.scale(x, y);
        m
    }

    /// Translate this matrix by given units
    pub fn translate(&mut self, x: f32, y: f32) {
        self.0[0][2] += x;
        self.0[1][2] += y;
    }

    /// Return a translated clone of this matrix
    pub fn translated(&self, x: f32, y: f32) -> Matrix {
        let mut m = self.clone();
        m.translate(x, y);
        m
    }

    /// Rotate this matrix around the origin by radian angle `a`
    pub fn rotate(&mut self, a: f32) {
        self.multiply(&Matrix::from(&[
            [a.cos(), -a.sin(), 0.0],
            [a.sin(), a.cos(), 0.0],
            [0.0, 0.0, 1.0],
        ]))
    }

    /// Return a rotated clone of this matrix
    pub fn rotated(&self, a: f32) -> Matrix {
        let mut m = self.clone();
        m.rotate(a);
        m
    }

    /// Multiply this matrix by a given matrix
    pub fn multiply(&mut self, other: &Self) {
        // Dot multiplication, not sure if correct, i think it is
        let me = self.clone();
        for x in 0 .. 2 {
            for y in 0 .. 2 {
                self.0[x][y] = 
                    me.0[x][0] * other.0[0][y] + 
                    me.0[x][1] * other.0[1][y] + 
                    me.0[x][2] * other.0[2][y];
            }
        }
    }

    /// Return a multiplied clone of this matrix
    pub fn multiplied(&self, other: &Self) -> Matrix {
        let mut m = self.clone();
        m.multiply(other);
        m
    }

    pub fn raw(&self) -> [[f32; 3]; 3] {
        self.0
    }
}

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
    fn present(&mut self, vsync: bool);
}

/// Common trait for both Texture and Surface, where you can draw to
pub trait Target<R: Renderer> {
    /// Get the size of this target in pixels
    fn size(&self) -> Size;
    /// Set (clear) the target to some specific color
    fn set(&mut self, color: &Color);
    /// Draw a texture into this target, by transforming it with the provided
    /// matrix. 
    fn draw(&mut self, texture: &R::Texture, transform: &Matrix);
}