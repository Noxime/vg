extern crate image;
extern crate kea;
extern crate piston_window;

use kea::renderer::*;

use self::piston_window::{OpenGL, PistonWindow, WindowSettings};

pub struct PistonRenderer {
    window: PistonWindow,
}

impl PistonRenderer {
    pub fn new() -> PistonRenderer {
        PistonRenderer {
            window: WindowSettings::new("kea", [1280, 720])
                .opengl(OpenGL::V3_2)
                .samples(4)
                .exit_on_esc(true) // TODO
                .vsync(true)
                .build()
                .expect("Window creation failed"),
        }
    }
}

impl Renderer for PistonRenderer {
    const NAME: &'static str = "Piston (GL32)";
    type Texture = PistonTexture;
    type Surface = PistonWindow;

    fn surface(&mut self) -> &mut Self::Surface {
        &mut self.window
    }
}

pub struct PistonTexture();

impl kea::renderer::Texture<PistonRenderer> for PistonTexture {
    fn new(_size: &Size, _color: &Color) -> Self {
        unimplemented!()
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
    fn scale(&mut self, _size: &Size) {
        unimplemented!()
    }
}

impl kea::renderer::Target<PistonRenderer> for PistonTexture {
    fn size(&self) -> Size {
        unimplemented!()
    }
    fn set(&mut self, _color: &Color) {
        unimplemented!()
    }
    fn draw(&mut self, _texture: PistonTexture, _coords: Coordinate) {
        unimplemented!()
    }
}

impl kea::renderer::Surface<PistonRenderer> for PistonWindow {
    fn capture(&self) -> PistonTexture {
        unimplemented!()
    }
}

impl kea::renderer::Target<PistonRenderer> for PistonWindow {
    fn size(&self) -> Size {
        use piston_window::Window;
        let s = self.window.size();
        [s.width as _, s.height as _]
    }

    fn set(&mut self, color: &Color) {
        use piston_window::{clear, Event, Loop};

        // TODO: For some reason we need a render arg to draw, so wait for it
        loop {
            if let Some(Event::Loop(Loop::Render(args))) = self.next() {
                self.draw_2d(&Event::Loop(Loop::Render(args)), |_ctx, gfx| {
                    clear(*color, gfx);
                });
                return;
            }
        }
    }

    fn draw(&mut self, _texture: PistonTexture, _coords: Coordinate) {
        unimplemented!()
    }
}
