mod sprite_renderer;
pub use self::sprite_renderer::SpriteRenderer;
mod sound_player;
pub use self::sound_player::SoundPlayer;

use graphics::Renderer;
use std::any::Any;

pub trait Component {
    // Rust typesystem is so fucking cool, but also takes a while to wrap your
    // head around. Also, this might not be optimal but its the best way I found
    // so far.
    // You most likely want to implement this as
    // `fn as_any(&self) -> &dyn Any { self as &Any }`
    fn as_any(&self) -> &dyn Any;

    fn render_init(&mut self, _: &mut Renderer) {}
    fn render(&mut self, _: &mut Renderer) {}
    fn render_destroy(&mut self, _: &mut Renderer) {}
}
