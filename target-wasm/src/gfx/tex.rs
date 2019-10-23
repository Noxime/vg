use kea::renderer::*;

pub struct Tex {
    
}

impl kea::renderer::Texture<super::Gfx> for Tex {
    fn new(r: &mut super::Gfx, size: &Size, color: &Color) -> Self {
        unimplemented!()
    }

    fn from_data(r: &mut super::Gfx, size: &Size, data: &Vec<Color>) -> Self {
        unimplemented!()
    }

    fn clone(&self) -> Self {
        unimplemented!()
    }
}


impl kea::renderer::Target<super::Gfx> for Tex {
    fn size(&self) -> Size {
        unimplemented!()
    }

    fn set(&mut self, color: &Color) {
        unimplemented!()
    }

    fn draw(&mut self, texture: &Self, shading: &Shading, view: &View, transform: &Transform) {
        unimplemented!()
    }
}