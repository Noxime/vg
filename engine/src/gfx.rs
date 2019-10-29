use crate::{Color, Matrix, Size};

pub trait Gfx: Target {
    fn api(&self) -> String;
    fn texture(&self, source: impl Source) -> Texture;
    fn present(&mut self) -> Present;
}

pub trait Target {
    fn size(&self) -> Size;
    fn fill(&mut self, color: Color);
    fn draw(&mut self, texture: &Texture, matrices: &[Matrix]);
}

#[crate::async_trait]
pub trait Source {
    async fn get(&self) -> (Size, Vec<Color>);
    /// Does the source have new data, should the texture refresh its contents
    ///
    /// For example, see `StreamingSource`
    async fn changed(&self) -> bool;
}

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

pub trait TextureTrait: Target {

}

pub struct Present(#[doc(hidden)] pub Box<dyn std::future::Future<Output = ()>>);
