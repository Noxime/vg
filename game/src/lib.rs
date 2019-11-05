use vg::renderer::{Renderer, Surface, Target, Texture};
// use audio::Clip;
use vg::*;

const ASSETS: assets::Assets = asset_pack!("assets.vgpack");

pub async fn run<A: Api>(mut api: A) {
    println!("assets.pack contains {} bytes of data", ASSETS.size());

    let (size, pixels) = vg::assets::png(
        ASSETS
            .assets("textures")
            .expect("tex")
            .binary("duburrito.png")
            .expect("duburrito"),
    )
    .expect("invalid png");
    let tex =
        <<A as Api>::R as Renderer>::Texture::from_data(api.renderer(), &size, &pixels);

    let t = A::T::now();

    while !api.exit() {
        api.poll();
        // let id = api.input().default().unwrap();

        api.renderer().surface().set(&[0.65, 0.87, 0.91, 1.0]);
        api.renderer().surface().draw(
            &tex,
            &Default::default(),
            &vg::renderer::View {
                x: 0.0,
                y: 0.0,
                rotation: t.elapsed(),
                scale: vg::renderer::Scale::Vertical(1.0),
                pixels_per_unit: 256.0,
            },
            &Default::default(),
        );

        api.renderer().surface().present(true).await;
    }
}
