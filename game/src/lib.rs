extern crate kea;

use kea::renderer::{Color, Matrix, Size, Surface, Target, Texture};
use kea::*;

use std::time::Instant;

macro_rules! load_image {
    ($path: expr) => {{
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
    }};
}

struct Camera {
    aspect: f32,
    x: f32,
    y: f32,
    zoom: f32,
}

struct Transform {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Transform {
    fn matrix(&self, camera: &Camera) -> Matrix {
        Matrix::identity()
            .translated(self.x / camera.zoom, self.y / camera.zoom)
            .scaled(self.w, self.h)
            .translated(-camera.x / camera.zoom, -camera.y / camera.zoom)
            .scaled(1.0 / camera.zoom, 1.0 / camera.zoom * camera.aspect)
    }
}

fn random() -> f32 {
    1.0
}

pub fn game<P, R, I>(mut api: EngineApi<P, R, I>)
where
    P: PlatformApi,
    R: renderer::Renderer,
    I: input::Input,
{
    {
        let [w, h] = api.renderer.surface().size();
        api.platform.print(&format!("Renderer is: {}", R::NAME));
        api.platform.print(&format!("Window is: {}x{}", w, h));
    }

    let grass_tex = {
        let (w, h, i) = load_image!("../assets/textures/grass.png");
        R::Texture::from_data(&mut api.renderer, &[w, h], &i)
    };

    let cloud_tex = {
        let (w, h, i) = load_image!("../assets/textures/cloud.png");
        R::Texture::from_data(&mut api.renderer, &[w, h], &i)
    };

    let cow = [
        {
            let (w, h, i) = load_image!("../assets/textures/cow_0.png");
            R::Texture::from_data(&mut api.renderer, &[w, h], &i)
        },
        {
            let (w, h, i) = load_image!("../assets/textures/cow_1.png");
            R::Texture::from_data(&mut api.renderer, &[w, h], &i)
        },
    ];

    let mut last = Instant::now();
    let mut time = 0.0;

    let mut camera = Camera {
        x: 0.0,
        y: 4.0,
        aspect: 1.0,
        zoom: 4.0,
    };

    let mut clouds = vec![
        (-5.0, 2.0, 0.5),
        (-5.0, 3.0, 0.38),
        (-5.0, 6.0, 0.83),
        (-5.0, 4.0, 0.12),
    ];

    {
        let c = api.input.controller(&api.input.default()).unwrap();
        api.platform.print(&format!("Using controller: `{}` ({:?})", c.info.name, c.info.power));
    }

    while !api.platform.exit() {
        let controller = {
            let id = api.input.default();
            api.input.controller(&id).unwrap()
        };
        api.poll();

        let size = api.renderer.surface().size();
        camera.aspect = size[0] as f32 / size[1] as f32;
        let delta = last.elapsed().subsec_nanos() as f32 / 1_000_000_000.0;
        time += delta;
        last = Instant::now();
        // println!("FPS: {:.2}", 1.0 / delta);

        api.renderer.surface().set(&[0.65, 0.87, 0.91, 1.0]);

        for i in 0..8 {
            api.renderer.surface().draw(
                &grass_tex,
                &Transform {
                    x: -3.5 + i as f32,
                    y: 0.0,
                    w: 1.0,
                    h: 1.0,
                }
                .matrix(&camera),
            );
        }



        api.renderer.surface().draw(
            &cloud_tex,
            &Transform {
                x: controller.right_joy.x + 2.0,
                y: controller.right_joy.y + 5.0,
                w: controller.left_shoulder.bumper.value() + 1.0,
                h: controller.right_shoulder.bumper.value() * 0.5 + 0.5,
            }
            .matrix(&camera),
        );

        let frame = {
            if time % 1.0 > 0.5 {
                1
            } else {
                0
            }
        };

        let frame = if controller.buttons.down.active() { 1 } else { 0 };

        api.renderer.surface().draw(
            &cow[frame],
            &Transform {
                x: controller.left_joy.x,
                y: controller.left_joy.y,
                w: 3.0,
                h: 5.0,
            }
            .matrix(&camera),
        );

        api.renderer.surface().present(true);
    }
}
