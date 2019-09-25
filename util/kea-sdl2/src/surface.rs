use kea::renderer::{Size, Color, View, Transform, Shading};

pub struct Surface {
    pub(crate) canvas: sdl2::render::WindowCanvas,
}

impl kea::renderer::Surface<super::Renderer> for Surface {
    fn capture(&self) -> super::Texture {
        unimplemented!()
    }

    fn present(&mut self, vsync: bool) {
        // theres something cursed about this... for some reason this only works if you get the errors before and after present
        match sdl2::get_error().as_str() {
            e if !e.is_empty() => println!("Error: {}", e),
            _ => ()
        }

        self.canvas.present();

        match sdl2::get_error().as_str() {
            e if !e.is_empty() => println!("Error: {}", e),
            _ => ()
        }
    }
}

impl kea::renderer::Target<super::Renderer> for Surface {
    fn size(&self) -> Size {
        let (w, h) = self.canvas.output_size().expect("Failed to get surface size");
        [w as usize, h as usize]
    }

    fn set(&mut self, color: &Color) {
        self.canvas.set_draw_color(sdl2::pixels::Color::RGBA((color[0] * 255.0) as u8, (color[1] * 255.0) as u8, (color[2] * 255.0) as u8, (color[3] * 255.0) as u8));
        self.canvas.clear();
    }

    fn draw(&mut self, texture: &super::Texture, shading: &Shading, view: &View, transform: &Transform) {
        unimplemented!()
    }
}