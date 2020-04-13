use crate::{gfx::Target, Color, Matrix, Size};

mod source;
pub use source::{RawSource, Source};

pub struct Texture {
    pub sampling: Sampling,
    // TODO: Figure out a better data structure and get rid of locks
    #[cfg_attr(not(feature = "dev-docs"), doc(hidden))]
    pub tex: Box<dyn TextureTrait>,
}

impl Texture {
    pub(crate) fn new(tex: Box<dyn TextureTrait>) -> Texture {
        Texture {
            tex,
            sampling: Sampling::Nearest,
        }
    }

    pub fn sampling(&self) -> Sampling {
        self.sampling
    }

    pub fn set_sampling(&mut self, sampling: Sampling) {
        self.sampling = sampling
    }
}

impl Target for Texture {
    fn size(&self) -> Size {
        self.tex.size()
    }
    fn fill(&mut self, color: Color) {
        self.tex.fill(color)
    }
    fn draw(&mut self, texture: &Texture, matrices: &[Matrix]) {
        self.tex.draw(texture, matrices)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Sampling {
    Linear,
    Nearest,
}

#[cfg_attr(not(feature = "dev-docs"), doc(hidden))] 
/// Represents a texture thats been uploaded to the GPU
/// 
/// Obtained through [`Gfx::upload_texture`]
pub trait TextureTrait: Target {}

// impl std::any::Any for dyn TextureTrait {}
