#[macro_use]
extern crate log;
extern crate kea;

use kea::entity::*;
use kea::scene::*;
use kea::vectors::*;
use kea::component::Component;
use kea::*;

enum SceneName {
    Main,
}

fn main() {
    run(Vec2::new(800, 600), "Kea Game".into(), &mut scene_loader, SceneName::Main);
}

fn scene_loader(scene: SceneName) -> Scene {
    match scene {
        SceneName::Main => Scene::empty().with_entity(Entity::empty()),
    }
}

struct TestComponent;
impl Component for TestComponent {
    fn initialize(&mut self) {
        info!("Called initialize on component");
    }
    fn destroy(&mut self) {
        info!("Called destory on component");
    }
}