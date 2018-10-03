#[macro_use]
extern crate log;
extern crate kea;

use kea::{components::*, entity::*, scene::*, vectors::*, *};

enum SceneName {
    Main,
}

fn main() {
    run(
        Vec2::new(800, 600),
        "Kea Game".into(),
        &mut scene_loader,
        SceneName::Main,
    );
}

fn scene_loader(scene: SceneName) -> Scene {
    match scene {
        SceneName::Main => Scene::empty().with_entity(
            Entity::empty()
                .with_component(Box::new(TestComponent))
                .with_component(Box::new(SpriteRenderer::new(include_bytes!(
                    "../assets/textures/test.png"
                )))).with_component(Box::new(SoundPlayer::new(
                    "/home/noxim/Music/fuck.wav",
                ))),
        ),
    }
}

struct TestComponent;
impl Component for TestComponent {}
