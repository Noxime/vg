use crate::{gfx, Color, Size};
use async_trait::async_trait;

#[async_trait]
pub trait Source: Send {
    async fn load(&self) -> (Vec<Color>, Size);
}

#[async_trait]
impl Source for Color {
    async fn load(&self) -> (Vec<Color>, Size) {
        (vec![*self], Size::new(1, 1))
    }
}

pub struct Texture {
    pub(crate) source: Box<dyn Source>,
    pub(crate) tex: gfx::Tex,
}
