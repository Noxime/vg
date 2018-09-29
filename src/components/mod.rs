mod sprite_renderer;
pub use self::sprite_renderer::SpriteRenderer;
use graphics::*;

pub trait Component {
    fn initialize(&mut self) {}
    fn destroy(&mut self) {}

    fn render_init(&mut self, data: &mut APIData) {}
    fn render(&mut self, data: &mut APIData) {}
    fn render_destroy(&mut self, data: &mut APIData) {}
}
