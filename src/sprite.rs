use api::*;

pub struct Sprite {
}

impl Sprite {
    pub fn new(filename: &str) -> Result<Self, ()> {

        Ok(Sprite {})
    }

    pub fn draw(&self, shader: &Shader, api: &Box<GfxApi>) {
        api.debug_draw_vertices(shader, &vec![
            (-0.5, -0.5, 0.0),
            ( 0.5, -0.5, 0.0),
            (-0.5,  0.5, 0.0),
        ])
    }
}