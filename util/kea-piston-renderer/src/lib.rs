extern crate kea;
extern crate piston_window;

use kea::renderer::*;
use kea::{Transform, Sprite};


use self::piston_window::*;

use std::rc::Rc;
use std::borrow::Borrow;

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
                .build().expect("Window creation failed")
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
        let (x, y) = (w / 2.0, h / 2.0);

        self.window.draw_2d(&Event::Loop(Loop::Render(args)), |ctx, gfx| {
            clear(base, gfx);

            let transform = ctx.transform
                .trans(x, y)
                .zoom(h * 0.2);

            for layer in layers {
                for sprite in &layer.sprites {
                    image(sprite.tex.0.borrow(), transform.trans((sprite.trans.pos.0 * layer.parallax)  as _, (sprite.trans.pos.1 * layer.parallax) as _), gfx);
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
        let settings = TextureSettings::new();

        let mut converted = vec![];
        for col in data {
            for pix in *col {
                converted.push((pix[0] * 255.0) as u8);
            }
        }

        let tex = G2dTexture::from_memory_alpha(factory, &converted, 4, 4, &settings).expect("Texture create failed");

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
