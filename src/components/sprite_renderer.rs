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
    fn render(&mut self) -> Option<DrawCall> {
        debug!("Rendering SpriteRenderer");
        Some(DrawCall::empty())
    }
}