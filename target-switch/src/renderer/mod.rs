use kea::renderer::*;
use kea::{Transform, Sprite};

use libnx_rs::libnx;

extern crate piston_window;
extern crate libnx_rs_window;

use self::piston_window::*;
use self::libnx_rs_window::NxGlWindow;

use std::rc::Rc;
use std::borrow::Borrow;

pub struct SwitchRenderer {
    window: PistonWindow<NxGlWindow>,
}

impl SwitchRenderer {
    pub fn new() -> SwitchRenderer {
        SwitchRenderer {
            window: WindowSettings::new("kea", [1280, 720])
                .opengl(OpenGL::V3_2)
                .samples(4)
                .exit_on_esc(true) // TODO
                .vsync(true)
                .build().expect("Window creation failed")
        }
    }
}


impl Renderer for SwitchRenderer {
    const NAME: &'static str = "Piston (GL32)";

    type Texture = SwitchTexture;
    type Layer = SwitchLayer;
    type Sprite = SwitchSprite;

    fn render(&mut self, base: Color, layers: &[Self::Layer]) {
        let event;
        loop {
            if let Some(Event::Loop(Loop::Render(args))) = self.window.next() {
                event = Event::Loop(Loop::Render(args));
                break;
            }
        }

        self.window.draw_2d(&event, |ctx, gfx| {
            clear(base, gfx);
            for layer in layers {
                for sprite in &layer.sprites {
                    image(sprite.tex.0.borrow(), ctx.transform, gfx)
                }
            }
        });
    }

    fn layer(&mut self, parallax: f32, sprites: &[&Self::Sprite]) -> Self::Layer {
        SwitchLayer {
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

        let tex = G2dTexture::from_memory_alpha(factory, &converted, data.len() as _, data[0].len() as _, &settings).expect("Texture create failed");

        SwitchTexture(Rc::new(tex))
    }

    fn sprite(&mut self, transform: Transform, texture: Self::Texture) -> Self::Sprite {
        SwitchSprite {
            trans: transform,
            tex: texture,
        }
    }
}

pub struct SwitchTexture(Rc<G2dTexture>);
impl Clone for SwitchTexture {
    fn clone(&self) -> SwitchTexture {
        SwitchTexture(Rc::clone(&self.0))
    }
}

pub struct SwitchLayer {
    parallax: f32,
    sprites: Vec<SwitchSprite>,
}

#[derive(Clone)]
pub struct SwitchSprite {
    trans: Transform,
    tex: SwitchTexture,
}

impl Sprite for SwitchSprite {
    fn transform(&self) -> &Transform {
        &self.trans
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.trans
    }
}