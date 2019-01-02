use kea::renderer::*;

use libnx_rs::libnx;

extern crate piston_window;
extern crate libnx_rs_window;

use self::piston_window::{OpenGL, PistonWindow, WindowSettings};
use self::libnx_rs_window::NxGlWindow;

pub struct SwitchRenderer {
    window: PistonWindow<NxGlWindow>,
}

impl SwitchRenderer {
    pub fn new() -> SwitchRenderer {
        SwitchRenderer {
            window: WindowSettings::new("kea", [1280, 720])
                .opengl(OpenGL::V3_2)
                .samples(4)
                .exit_on_esc(true) // TODO
                .vsync(true)
                .build().expect("Window creation failed")
        }
    }
}


impl Renderer for SwitchRenderer {
    const NAME: &'static str = "Piston (GL32)";
    type Texture = SwitchTexture;
    type Surface = PistonWindow<NxGlWindow>;

    fn surface(&mut self) -> &mut Self::Surface {
        &mut self.window
    }
}

pub struct SwitchTexture();

impl kea::renderer::Texture<SwitchRenderer> for SwitchTexture {
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

impl kea::renderer::Target<SwitchRenderer> for SwitchTexture {
    fn size(&self) -> Size {
        unimplemented!()
    }
    fn set(&mut self, _color: &Color) {
        unimplemented!()
    }
    fn draw(&mut self, _texture: SwitchTexture, _coords: Coordinate) {
        unimplemented!()
    }
}

impl kea::renderer::Surface<SwitchRenderer> for PistonWindow<NxGlWindow> {
    fn capture(&self) -> SwitchTexture {
        unimplemented!()
    }
}

impl kea::renderer::Target<SwitchRenderer> for PistonWindow<NxGlWindow> {
    fn size(&self) -> Size {
        use self::piston_window::Window;
        let s = self.window.size();
        [s.width as _, s.height as _]
    }

    fn set(&mut self, color: &Color) {
        use self::piston_window::{clear, Event, Loop};

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

    fn draw(&mut self, _texture: SwitchTexture, _coords: Coordinate) {
        unimplemented!()
    }
}