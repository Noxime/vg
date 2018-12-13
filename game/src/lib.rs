extern crate kea;

use kea::*;

pub fn game(mut api: EngineApi<impl PlatformApi, impl renderer::Renderer>) {
    api.platform.print("Hello world");
    let mut b = false;
    let tex = api.renderer.texture(&[
        &[[0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]],
        &[[1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 1.0]],
        &[[0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]],
        &[[1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 1.0]],
    ]);

    let sprite = api.renderer.sprite(Transform::default(), tex);

    loop {
        b = !b;
        use kea::renderer::*;
        let foreground = api.renderer.layer(1.0, &[&sprite]);
        api.renderer.render([0.0; 4], &[foreground]);
    }
}