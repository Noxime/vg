#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate pretty_env_logger;
extern crate winit;

pub mod components;
pub mod entity;
pub mod graphics;
pub mod audio;
pub mod input;
pub mod scene;
pub mod vectors;
use components::Component;
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
    pretty_env_logger::init();
    audio::init();
    let mut i = input::init();

    let apis = graphics::supported();
    debug!("APIs supported: {}", apis.len());
    for (api, support) in &apis {
        debug!("  {:?} = {}", api, support);
    }
    let mut events = match graphics::create(size, title, &apis[0].0) {
        Ok(v) => v,
        Err(why) => {
            error!("Graphics pipeline creation failed: {:?}", why);
            panic!()
        }
    };

    debug!("Loading start scene");
    let mut scene = scene_loader(start_scene);
    graphics::render_init(&mut scene);

    debug!("Entering main loop");
    'main: loop {
        let mut close = false;
        events.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                // window events
                match event {
                    WindowEvent::CloseRequested => close = true,
                    winit::WindowEvent::Resized(dims) => {
                        debug!("resized to {:?}", dims);
                        graphics::resize(Vec2::new(
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

        input::events(&mut i);

        // debug!("PRECRASH");
        graphics::pre_render();
        graphics::render(&mut scene);
        graphics::post_render();
    }

    graphics::render_destroy(&mut scene);

    info!("Kea shutdown");
}
