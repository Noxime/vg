use Transform;
use Sprite;

// Frame 
// -> Layer
//    -> Sprite
//    -> Sprite
// -> Layer
//    -> Sprite
// -> Layer


// player = api.sprite()
// wall = api.sprite()
// player.move(5, 2)
// wall.scale(50, 50)
// api.render(Color::BLACK, &[
//     api.layer(1.0, &[player]),
//     api.layer(0.2, &[wall]),
// ]);


pub type Color = [f32; 4];

pub trait Renderer {
    /// A user friendly name of our rendering engine
    const NAME: &'static str;

    type Texture;
    type Layer;
    type Sprite: Sprite;

    /// Create a new frame object to render to
    fn render(&mut self, base: Color, layers: &[Self::Layer]);
    /// Create a new renderlayer from sprites
    fn layer(&mut self, parallax: f32, sprites: &[&Self::Sprite]) -> Self::Layer;
    /// Create a new texture from raw color data, that can be assigned to sprites
    fn texture(&mut self, data: &[&[Color]]) -> Self::Texture;
    fn sprite(&mut self, transform: Transform, texture: Self::Texture) -> Self::Sprite;
}