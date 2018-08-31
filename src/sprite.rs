use api::*;
use arch;

pub struct Sprite {
    tex: Texture,
}

impl Sprite {
    pub fn new(api: &Box<GfxApi>, filename: &str) -> Result<Self, ()> {
        let (width, height, data) = arch::load_png(filename)?;
        let tex = api.upload_texture(width, height, data, false);

        Ok(Sprite { tex })
    }

    pub fn draw(&self, shader: &Shader, api: &Box<GfxApi>) {
        api.debug_draw_vertices(
            shader,
            &vec![
                ((-0.5, -0.5, 0.0), (0.0, 0.0)),
                ((0.5, -0.5, 0.0), (1.0, 0.0)),
                ((-0.5, 0.5, 0.0), (0.0, 1.0)),
                ((0.5, 0.5, 0.0), (1.0, 1.0)),
                ((0.5, -0.5, 0.0), (1.0, 0.0)),
                ((-0.5, 0.5, 0.0), (0.0, 1.0)),
            ],
            Some(&self.tex),
        )
    }
}
