#[macro_use] extern crate log;
extern crate kea;

use kea::*;
use kea::vectors::*;

fn main() {
    init();

    let api = {
        info!("Supported API's:");
        let apis = graphics::supported();
        for (i, (api, support)) in apis.iter().enumerate() {
            info!("  {}: {:?} = {}", i, api, support);
        }
        let (api, _) = apis.get(0).expect("No APIs available");
        *api
    };
    info!("Using: {:?}", api);

    // graphics::create(Vec2::new(800, 600), "Kea".into(), &api);
    graphics::create(Vec2::new(800, 600), "Kea".into(), &api);
}