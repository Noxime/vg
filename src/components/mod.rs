mod sprite_renderer;
pub use self::sprite_renderer::SpriteRenderer;
use graphics::*;

pub trait Component {
    fn initialize(&mut self) {}
    fn destroy(&mut self) {}

    fn render(&mut self) -> Option<DrawCall> { None }
}
