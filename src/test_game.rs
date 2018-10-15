extern crate log;
extern crate kea;

use kea::{components::*, entity::*, scene::*, vectors::*, *};
use std::any::Any;

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
                .with(TestComponent)
                .with(SpriteRenderer::new(include_bytes!(
                    "../assets/textures/test.png"
                ))).with(SoundPlayer::new("/home/noxim/Music/fuck.wav")),
        ),
    }
}

struct TestComponent;
impl Component for TestComponent {
    fn as_any(&self) -> &dyn Any { self as &Any }
}
