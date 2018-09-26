#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;
extern crate pretty_env_logger;
extern crate winit;

pub mod graphics;
pub mod vectors;
pub mod scene;
use vectors::*;

use winit::*;

/// initialize static parts of kea.
/// This should be the first function you call
pub fn run<T>(size: Vec2<usize>, title: String, scene_loader: &FnMut(T) -> scene::Scene) {
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


    'main: loop {
        let mut close = false;
        events.poll_events(|event| if let Event::WindowEvent { event, .. } = event {
            // window events
            match event {
                WindowEvent::CloseRequested => close = true,
                _ => (),
            }
        } else if let Event::DeviceEvent { event, .. } = event {
            // device events
        });
        if close { break 'main; }

    }

    info!("Kea initialized");
}