#[macro_use] extern crate log;
extern crate kea;

use kea::*;
use kea::vectors::*;
use kea::scene::*;

enum SceneName {
    Main,
}

fn main() {
    run(Vec2::new(800, 600), "Kea Game".into(), &scene_loader);
}

fn scene_loader(scene: SceneName) -> Scene {
    match scene {
        SceneName::Main => Scene::new(),
    }
}