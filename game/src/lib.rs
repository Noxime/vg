use kea::renderer::{Renderer, Texture, Surface, Target};
// use audio::Clip;
use kea::*;

const ASSETS: assets::Assets = asset_pack!("assets.keapack");

pub async fn run<A: Api>(mut api: A) {
    println!("assets.pack contains {} bytes of data", ASSETS.size());

    let tex = <<A as Api>::R as Renderer>::Texture::new(api.renderer(), &[1, 1], &[0.5, 0.2, 0.8, 1.0]);

    while !api.exit() {
        api.poll();
        // let id = api.input().default().unwrap();

        api.renderer().surface().set(&[0.65, 0.87, 0.91, 1.0]);
        api.renderer().surface().draw(&tex, &Default::default(), &kea::renderer::View {
            x: 0.5,
            y: 0.5,
            rotation: 0.0,
            scale: kea::renderer::Scale::Horizontal(1.0),
            pixels_per_unit: 1.0,
        }, &Default::default());

        api.renderer().surface().present(true).await;
    }
}
