extern crate kea;

use kea::*;
use kea::renderer::{Target, Texture, Surface, Matrix};

use std::time::Instant;

pub fn game<P, R>(mut api: EngineApi<P, R>)
where
    P: PlatformApi,
    R: renderer::Renderer,
{
    {
        let [w, h] = api.renderer.surface().size();
        api.platform.print(&format!("Renderer is: {}", R::NAME));
        api.platform.print(&format!("Window is: {}x{}", w, h));
    }

    let tex = R::Texture::new(&mut api.renderer, &[64, 64], &[rand::random(), rand::random(), rand::random(), 1.0]);
    let items: Vec<Matrix> = (0 .. 2000).map(|_| {
        let mut m = Matrix::identity();
        m.translate(rand::random::<f32>() * 2.0 - 1.0, rand::random::<f32>() * 2.0 - 1.0);
            m.scale(rand::random(), rand::random());
        m
    }).collect();

    let mut f: f32 = 0.0;
    let mut last = Instant::now();

    loop {
        let delta = last.elapsed().subsec_nanos() as f32 / 1_000_000_000.0;
        last = Instant::now();
        f += delta;
        println!("FPS: {:.2}", 1.0 / delta);

        api.renderer.surface().set(&[0.0, 0.0, 0.0, 1.0]);

        for m in &items {
            let mut m = m.clone();
            m.rotate(f);
            api.renderer.surface().draw(&tex, &m);
        }
        api.renderer.surface().present(true);
    }
}
