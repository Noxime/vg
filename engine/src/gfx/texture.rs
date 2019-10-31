use crate::{gfx::Target, Color, Matrix, Size};

mod source;
pub use source::{Source, RawSource, LazySource};

pub struct Texture {
    sampling: Sampling,
    inner: Box<dyn TextureTrait>,
}

impl Texture {
    pub fn sampling(&self) -> Sampling {
        self.sampling
    }

    pub fn set_sampling(&mut self, sampling: Sampling) {
        self.sampling = sampling
    }
}

impl Target for Texture {
    fn size(&self) -> Size {
        self.inner.size()
    }
    fn fill(&mut self, color: Color) {
        self.inner.fill(color)
    }
    fn draw(&mut self, texture: &Texture, matrices: &[Matrix]) {
        self.inner.draw(texture, matrices)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Sampling {
    Linear,
    Nearest,
}

pub trait TextureTrait: Target {}
