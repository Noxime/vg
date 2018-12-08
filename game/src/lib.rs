extern crate kea;
extern crate log;

use kea::{components::*, entity::*, scene::*};
use std::any::Any;

pub enum SceneName {
    Main,
}

pub const INITIAL: SceneName = SceneName::Main;
pub fn loader(scene: SceneName) -> Scene {
    match scene {
        SceneName::Main => Scene::empty().with_entity(
            Entity::empty()
                .with(TestComponent)
                .with(SpriteRenderer::new(include_bytes!(
                    "../assets/textures/test.png"
                )))
        ),
    }
}

struct TestComponent;
impl Component for TestComponent {
    fn as_any(&self) -> &dyn Any { self as &Any }
    fn as_any_mut(&mut self) -> &mut dyn Any { self as &mut Any }
}
