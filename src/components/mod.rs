mod sprite_renderer;
pub use self::sprite_renderer::SpriteRenderer;

use graphics::Renderer;
use std::any::Any;
use entity::Entity;

pub trait Component {
    // Rust typesystem is so fucking cool, but also takes a while to wrap your
    // head around. Also, this might not be optimal but its the best way I found
    // so far.
    // You most likely want to implement this as
    // `fn as_any(&self) -> &dyn Any { self as &Any }`
    // `fn as_any_mut(&mut self) -> &mut dyn Any { self as &mut Any }`
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn update(&mut self, _: &mut Entity) {}

    fn render_init(&mut self, _: &mut Renderer) {}
    fn render(&mut self, _: &mut Renderer) {}
    fn render_destroy(&mut self, _: &mut Renderer) {}
}
