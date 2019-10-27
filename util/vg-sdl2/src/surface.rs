use vg::renderer::{Size, Color, View, Transform, Shading};

pub struct Surface {
    pub(crate) canvas: sdl2::render::WindowCanvas,
}

struct Present(usize);
impl std::future::Future for Present {
    type Output = ();

    fn poll(mut self: std::pin::Pin<&mut Present>, _ctx: &mut std::task::Context<'_>) 
        -> std::task::Poll<()>
    {
        if self.0 == 0 {
            println!("Ready");
            std::task::Poll::Ready(())
        } else {
            println!("Pending, {} left", self.0);
            self.0 -= 1;
            std::task::Poll::Pending
        }
    }
}

impl vg::renderer::Surface<super::Renderer> for Surface {
    fn capture(&self) -> super::Texture {
        unimplemented!()
    }

    fn present(&mut self, vsync: bool)  -> Box<dyn std::future::Future<Output=()> + Unpin> {
        // theres something cursed about this... for some reason this only works if you get the errors before and after present
        match sdl2::get_error().as_str() {
            e if !e.is_empty() => println!("Error (before): {}", e),
            _ => ()
        }

        self.canvas.present();

        match sdl2::get_error().as_str() {
            e if !e.is_empty() => println!("Error (after): {}", e),
            _ => ()
        }

        Box::new(Present(5))
    }
}

impl vg::renderer::Target<super::Renderer> for Surface {
    fn size(&self) -> Size {
        let (w, h) = self.canvas.output_size().expect("Failed to get surface size");
        [w as usize, h as usize]
    }

    fn set(&mut self, color: &Color) {
        println!("Pre error: {}", sdl2::get_error());
        self.canvas.set_draw_color(sdl2::pixels::Color::RGBA((color[0] * 255.0) as u8, (color[1] * 255.0) as u8, (color[2] * 255.0) as u8, (color[3] * 255.0) as u8));
        println!("Post error: {}", sdl2::get_error());
        self.canvas.clear();
        println!("End error: {}", sdl2::get_error());
    }

    fn draw(&mut self, texture: &super::Texture, shading: &Shading, view: &View, transform: &Transform) {
        unimplemented!()
    }
}