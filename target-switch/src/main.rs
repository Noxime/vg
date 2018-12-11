

extern crate libnx_rs;

extern crate kea;
extern crate game;

use kea::{PlatformApi};

mod renderer;

struct SwitchApi;
impl PlatformApi for SwitchApi {
    fn print(&self, s: &str) { println!("{}", s); }
}

pub fn main() {
    kea::run(SwitchApi, renderer::SwitchRenderer::new(), &game::game);
    // // Construct the window.
    // let mut window: PistonWindow<NxGlWindow> = 
    //     WindowSettings::new("", [1280, 720])
    //         .opengl(OpenGL::V3_2) // If not working, try `OpenGL::V2_1`.
    //         .samples(4)
    //         .exit_on_esc(true)
    //         .vsync(true)
    //         .build()
    //         .unwrap();

    // // Poll events from the window.
    // while let Some(event) = window.next() {
    //     window.draw_2d(&event, |_context, graphics| {
    //         piston_window::clear([0.5, 0.2, 0.8, 1.0], graphics);
    //     });
    // }
}