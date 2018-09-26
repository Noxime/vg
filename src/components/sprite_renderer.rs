use components::Component;
use graphics::*;

pub struct SpriteRenderer {

}

impl SpriteRenderer {
    pub fn new() -> SpriteRenderer {
        SpriteRenderer {

        }
    }
}

impl Component for SpriteRenderer {
    fn prepare_render(&mut self) {
        debug!("Creating SpriteRenderer graphics");
    }
    fn render(&mut self) {
        debug!("Rendering SpriteRenderer");
    }
    fn destroy_render(&mut self) {
        debug!("Destroying SpriteRenderer graphics");
    }
}