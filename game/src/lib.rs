extern crate kea;

use kea::renderer::{Matrix, Surface, Target, Texture, Color, Size};
use kea::*;

use std::time::Instant;

macro_rules! load_image {
    ($path: expr) => {
        {
            let raw = image::load_from_memory(include_bytes!($path))
                .unwrap()
                .to_rgba();
            (
                raw.width() as usize,
                raw.height() as usize,
                raw.pixels()
                    .map(|p| {
                        [
                            p.data[0] as f32 / 255.0,
                            p.data[1] as f32 / 255.0,
                            p.data[2] as f32 / 255.0,
                            p.data[3] as f32 / 255.0,
                        ]
                    })
                    .collect(),
            )
        }
    };
}

struct Camera {
    x: f32,
    y: f32,
    fov: f32,
}

impl Camera {
    fn matrix(&self, surface: &Size) -> Matrix {
        let aspect = surface[1] as f32 / surface[0] as f32;
        let mut m = Matrix::identity();
        m.translate(-self.x * aspect, -self.y);
        m.scale(aspect, 1.0);
        m.scale(1.0 / self.fov, 1.0 / self.fov);
        m
    }
}

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

    let img = load_image!("../assets/textures/duburrito.png");
    let tex = R::Texture::from_data(&mut api.renderer, &[img.0, img.1], &img.2);

    let mut f: f32 = 0.0;
    let mut last = Instant::now();

    let mut camera = Camera {
        x: 0.0,
        y: 0.0,
        fov: 1.0,
    };

    loop {
        let size = api.renderer.surface().size();
        let delta = last.elapsed().subsec_nanos() as f32 / 1_000_000_000.0;
        last = Instant::now();
        f += delta;
        println!("FPS: {:.2}", 1.0 / delta);

        api.renderer.surface().set(&[0.0, 0.0, 0.0, 1.0]);
        
        camera.x = f.sin();
        camera.y = f.cos();

        api.renderer.surface().draw(&tex, &camera.matrix(&size));
        api.renderer.surface().present(true);
    }
}
