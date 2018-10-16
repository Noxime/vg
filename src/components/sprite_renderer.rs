extern crate image as image_crate;

use components::Component;
use graphics::*;
use std::io::Cursor;

use std::{any::Any, mem::size_of};

pub struct SpriteRenderer {
    image: image_crate::RgbaImage,
}

impl SpriteRenderer {
    pub fn new(image_data: &[u8]) -> SpriteRenderer {
        SpriteRenderer {
            image: image_crate::load(
                Cursor::new(&image_data[..]),
                image_crate::PNG,
            ).unwrap()
            .to_rgba(),
        }
    }
}

impl Component for SpriteRenderer {
    fn as_any(&self) -> &dyn Any { self as &Any }

    fn render_init(&mut self, _: &mut Renderer) {}
    fn render(&mut self, _: &mut Renderer) {}
    fn render_destroy(&mut self, _: &mut Renderer) {}
}
