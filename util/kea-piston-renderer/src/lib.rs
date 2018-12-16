extern crate image;
extern crate kea;
extern crate piston_window;

use kea::renderer::*;
use kea::{Sprite, Transform};

use image::*;

use self::piston_window::*;

use std::borrow::Borrow;
use std::rc::Rc;

pub struct PistonRenderer {
    window: PistonWindow,
}

impl PistonRenderer {
    pub fn new() -> PistonRenderer {
        PistonRenderer {
            window: WindowSettings::new("kea", [1280, 720])
                .opengl(OpenGL::V3_2)
                .samples(4)
                .exit_on_esc(true) // TODO
                .vsync(true)
                .build()
                .expect("Window creation failed"),
        }
    }
}

impl Renderer for PistonRenderer {
    const NAME: &'static str = "Piston (GL32)";

    type Texture = PistonTexture;
    type Layer = PistonLayer;
    type Sprite = PistonSprite;

    fn render(&mut self, base: Color, layers: &[Self::Layer]) {
        let args;
        loop {
            if let Some(Event::Loop(Loop::Render(a))) = self.window.next() {
                args = a;
                break;
            }
        }

        let (w, h) = (args.width, args.height);

        self.window
            .draw_2d(&Event::Loop(Loop::Render(args)), |ctx, gfx| {
                clear(base, gfx);
                let ctx = ctx.reset();
                let transform = ctx.transform;
                let view = ctx.transform.scale(h / w, 1.0);

                for layer in layers {
                    for sprite in &layer.sprites {
                        let img: &G2dTexture = sprite.tex.0.borrow();
                        image(
                            img,
                            transform
                                // .zoom(h)
                                .trans(
                                    (sprite.trans.pos.0 * layer.parallax) as _,
                                    (sprite.trans.pos.1 * layer.parallax) as _,
                                )
                                .scale(1.0 / img.get_width() as f64, 1.0 / img.get_height() as f64)
                                .trans(
                                    img.get_width() as f64 / -2.0,
                                    img.get_height() as f64 / -2.0,
                                )
                                .prepend_transform(view),
                            gfx,
                        );
                    }
                }
            });
    }

    fn layer(&mut self, parallax: f32, sprites: &[&Self::Sprite]) -> Self::Layer {
        PistonLayer {
            parallax,
            sprites: sprites.iter().cloned().cloned().collect(),
        }
    }

    fn texture(&mut self, data: &[&[Color]]) -> Self::Texture {
        let factory = &mut self.window.factory;
        let settings = TextureSettings::new().filter(Filter::Nearest);

        let img = RgbaImage::from_fn(data.len() as _, data[0].len() as _, |x, y| {
            Rgba([
                (data[x as usize][y as usize][0] * 255.0) as u8,
                (data[x as usize][y as usize][1] * 255.0) as u8,
                (data[x as usize][y as usize][2] * 255.0) as u8,
                (data[x as usize][y as usize][3] * 255.0) as u8,
            ])
        });

        let tex = G2dTexture::from_image(factory, &img, &settings).expect("Texture create failed");

        PistonTexture(Rc::new(tex))
    }

    fn sprite(&mut self, transform: Transform, texture: Self::Texture) -> Self::Sprite {
        PistonSprite {
            trans: transform,
            tex: texture,
        }
    }
}

pub struct PistonTexture(Rc<G2dTexture>);
impl Clone for PistonTexture {
    fn clone(&self) -> PistonTexture {
        PistonTexture(Rc::clone(&self.0))
    }
}

pub struct PistonLayer {
    parallax: f32,
    sprites: Vec<PistonSprite>,
}

#[derive(Clone)]
pub struct PistonSprite {
    trans: Transform,
    tex: PistonTexture,
}

impl Sprite for PistonSprite {
    fn transform(&self) -> &Transform {
        &self.trans
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.trans
    }
}
