#![feature(core_intrinsics)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate emoji_logger;
extern crate winit;

pub mod audio;
pub mod components;
pub mod entity;
pub mod graphics;
pub mod input;
pub mod scene;
pub mod vectors;
use vectors::*;

use winit::*;

/// initialize static parts of kea.
/// This should be the first function you call
pub fn run<T>(
    size: Vec2<usize>,
    title: String,
    scene_loader: &mut FnMut(T) -> scene::Scene,
    start_scene: T,
) {
    emoji_logger::init();
    info!("Running Kea, version {}", env!("CARGO_PKG_VERSION"));
    audio::init();
    // let mut i = input::init();

    let mut window = graphics::Window::new(size, &title);
    let mut graphics = graphics::Renderer::from(&mut window);
    info!("Initialization complete");

    debug!("Loading start scene");
    let mut scene = scene_loader(start_scene);
    scene.render_init(&mut graphics);

    debug!("Entering main loop");
    'main: loop {
        let mut close = false;
        window.events.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                // window events
                match event {
                    WindowEvent::CloseRequested => close = true,
                    winit::WindowEvent::Resized(dims) => {
                        graphics.resize(Vec2::new(
                            dims.width as usize,
                            dims.height as usize,
                        ));
                    }
                    _ => (),
                }
            } else if let Event::DeviceEvent { event: _, .. } = event {
                // device events
            }
        });
        if close {
            break 'main;
        }

        // input::events(&mut i);

        // debug!("PRECRASH");
        scene.render(&mut graphics);
        graphics.present();
    }

    scene.render_destroy(&mut graphics);

    info!("Kea shutdown");

    // FIXME: Something in the graphics backend segfaults here
}
