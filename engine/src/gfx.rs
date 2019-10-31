use crate::{Color, Matrix, Size};

pub mod texture;
use texture::{Source, Texture};

pub trait Gfx: Target {
    fn api(&self) -> String;
    fn texture(&self, source: Box<dyn Source>) -> Texture;
    fn present(&mut self) -> Present;
}

pub trait Target {
    fn size(&self) -> Size;
    fn fill(&mut self, color: Color);
    fn draw(&mut self, texture: &Texture, matrices: &[Matrix]);
}

pub struct Present(#[doc(hidden)] pub Box<dyn std::future::Future<Output = ()>>);
