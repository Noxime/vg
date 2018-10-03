mod sprite_renderer;
pub use self::sprite_renderer::SpriteRenderer;
mod sound_player;
pub use self::sound_player::SoundPlayer;
use graphics::*;

pub trait Component {
    fn render_init(&mut self, data: &mut APIData) {}
    fn render(&mut self, data: &mut APIData) {}
    fn render_destroy(&mut self, data: &mut APIData) {}
}
