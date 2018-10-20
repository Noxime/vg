#![feature(core_intrinsics)]
#![feature(test)]
extern crate test;
use test::Bencher;

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate emoji_logger;
extern crate winit;

pub mod components;
pub mod entity;
pub mod graphics;
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

        scene.render(&mut graphics);
        graphics.present();
    }

    scene.render_destroy(&mut graphics);

    info!("Kea shutdown");

    // FIXME: Something in the graphics backend segfaults here
}

#[bench]
fn ecs_test(b: &mut Bencher) {
    let mut scene = scene::Scene::empty();
    #[derive(Default)]
    struct Position {
        x: f32,
        y: f32,
        z: f32,
    }
    use std::any::Any;
    impl components::Component for Position {
        fn as_any(&self) -> &dyn Any { self as &Any }
        fn as_any_mut(&mut self) -> &mut dyn Any { self as &mut Any }
    }
    #[derive(Default)]
    struct Velocity {
        x: f32,
        y: f32,
        z: f32,
    }
    impl components::Component for Velocity {
        fn as_any(&self) -> &dyn Any { self as &Any }
        fn as_any_mut(&mut self) -> &mut dyn Any { self as &mut Any }

        fn update(&mut self, parent: &mut entity::Entity) {
            if let Some(ref mut pos) = parent.get_mut::<Position>() {
                pos.x += self.x;
                pos.y += self.y;
                pos.z += self.z;
            }
        }
    }

    for _ in 0..1000 {
        scene.add_entity(
            entity::Entity::empty()
                .with(Position::default())
                .with(Velocity::default()),
        );
    }
    for _ in 0..9000 {
        scene.add_entity(
            entity::Entity::empty()
                .with(Position::default())
        );
    }

    b.iter(|| scene.update());
}
