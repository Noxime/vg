mod sprite_renderer;
pub use self::sprite_renderer::SpriteRenderer;
use graphics::*;

pub trait Component {
    fn initialize(&mut self) {}
    fn destroy(&mut self) {}

    fn prepare_render(&mut self) {}
    fn render(&mut self) {}
    fn destroy_render(&mut self) {}
}