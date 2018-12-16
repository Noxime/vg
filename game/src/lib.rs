extern crate kea;

use kea::*;

pub fn game<P, R>(mut api: EngineApi<P, R>)
where
    P: PlatformApi,
    R: renderer::Renderer,
{
    api.platform.print(&format!("Renderer is: {}", R::NAME));

    let tex = api.renderer.texture(&[
        &[[0.2, 0.2, 0.2, 1.0], [1.0, 1.0, 1.0, 1.0], [0.2, 0.2, 0.2, 1.0], [1.0, 1.0, 1.0, 1.0]],
        &[[1.0, 1.0, 1.0, 1.0], [0.2, 0.2, 0.2, 1.0], [1.0, 1.0, 1.0, 1.0], [0.2, 0.2, 0.2, 1.0]],
        &[[0.2, 0.2, 0.2, 1.0], [1.0, 1.0, 1.0, 1.0], [0.2, 0.2, 0.2, 1.0], [1.0, 1.0, 1.0, 1.0]],
    ]);

    let mut sprite = api.renderer.sprite(Transform::default(), tex);

    let tex = api.renderer.texture(&[
        &[[0.2, 0.2, 0.2, 0.0], [1.0, 1.0, 1.0, 1.0], [0.2, 0.2, 0.2, 1.0], [1.0, 1.0, 1.0, 1.0]],
        &[[1.0, 1.0, 1.0, 0.0], [0.2, 0.2, 0.2, 1.0], [1.0, 1.0, 1.0, 1.0], [0.2, 0.2, 0.2, 1.0]],
        &[[0.2, 0.2, 0.2, 0.0], [1.0, 1.0, 1.0, 1.0], [0.2, 0.2, 0.2, 1.0], [1.0, 1.0, 1.0, 1.0]],
        &[[1.0, 1.0, 1.0, 0.0], [0.2, 0.2, 0.2, 1.0], [1.0, 1.0, 1.0, 1.0], [0.2, 0.2, 0.2, 1.0]],
    ]);

    let mut sprite2 = api.renderer.sprite(Transform::default(), tex);

    let mut f: f32 = 0.0;

    loop {
        f += 0.05;
        sprite.transform_mut().pos = (f.sin(), 0.0);
        sprite2.transform_mut().pos = (f.cos(), f.sin());

        let background = api.renderer.layer(0.25, &[&sprite2]);
        let foreground = api.renderer.layer(1.0, &[&sprite]);
        api.renderer.render([0.0, 0.0, 0.0, 1.0], &[background, foreground]);
    }
}