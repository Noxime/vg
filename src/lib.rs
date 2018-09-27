#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate pretty_env_logger;
extern crate winit;

pub mod graphics;
pub mod scene;
pub mod entity;
pub mod vectors;
pub mod components;
use components::Component;
use vectors::*;

use winit::*;

/// initialize static parts of kea.
/// This should be the first function you call
pub fn run<T>(size: Vec2<usize>, title: String, scene_loader: &mut FnMut(T) -> scene::Scene, start_scene: T) {
    pretty_env_logger::init();

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

    debug!("Entering main loop");
    'main: loop {
        let mut close = false;
        events.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                // window events
                match event {
                    WindowEvent::CloseRequested => close = true,
                    _ => (),
                }
            } else if let Event::DeviceEvent { event, .. } = event {
                // device events
            }
        });
        if close {
            break 'main;
        }

        graphics::render(&mut scene);
    }

    info!("Kea initialized");
}
