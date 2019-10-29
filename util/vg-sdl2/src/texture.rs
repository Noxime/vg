use vg::renderer::{Size, Color, View, Transform, Shading};

pub struct Texture;

impl vg::renderer::Texture<super::Renderer> for Texture {
    fn new(r: &mut super::Renderer, size: &Size, color: &Color) -> Self {
        unimplemented!()
    }

    fn from_data(r: &mut super::Renderer, size: &Size, data: &Vec<Color>) -> Self {
        unimplemented!()
    }

    fn clone(&self) -> Self {
        unimplemented!()
    }
}


impl vg::renderer::Target<super::Renderer> for Texture {
    fn size(&self) -> Size {
        unimplemented!()
    }

    fn set(&mut self, color: &Color) {
        unimplemented!()
    }

    fn draw(&mut self, texture: &super::Texture, shading: &Shading, view: &View, transform: &Transform) {
        unimplemented!()
    }
}