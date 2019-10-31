//! # Overview
//! vg is a lightweight game engine / framework, intended to abstract all the
//! platform specific parts to building a 2D game.
//! 
//! # Example
//! ```rust
//! vg::asset!("textures/ferris.png", FERRIS_PNG);
//! 
//! #[vg::game]
//! async fn example_game(vg: Vg) {
//!     vg.title("Example game");
//! 
//!     let ferris_png = PngSource::new(FERRIS_PNG);
//!     let ferris = vg.texture(LazySource::new(Color::hex(0xFF00FF), ferris_png));
//!     
//!     loop {
//!         vg.fill(Color::BLACK);
//!         vg.draw(&ferris, Matrix::identity().scale(0.1).rotate(vg.now()));
//!         vg.present().await;
//!     }
//! }
//! ```

pub use async_trait::async_trait;

pub type Size = [usize; 2];
pub type Color = [f32; 4];
pub type Matrix = [f32; 16];
pub type Time = f64;

pub const EPOCH: Time = 0.0;

mod asset;
pub mod gfx;
pub use asset::Asset;

pub struct Vg(Box<dyn Api>);

pub trait Api: gfx::Gfx {
    /// set the window title
    fn title(&mut self, title: String);
    /// resize the window
    fn size(&mut self, size: Size);
    /// set the window icon
    fn icon(&mut self, icon: Icon);
    /// set fullscreen
    fn fullscreen(&mut self, fullscreen: bool);

    fn now(&self) -> Time;
    fn elapsed(&self, since: Time) -> std::time::Duration {
        std::time::Duration::from_secs_f64(self.now() - since)
    }

    /// load an asset from a path
    fn asset(&self, path: &str) -> Option<Asset>;
}

pub struct Icon;
