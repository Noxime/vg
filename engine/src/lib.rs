//! # Overview
//! vg is a lightweight game engine / framework, intended to abstract all the
//! platform specific parts to building a 2D game.
//!
//! # Example
//! ```rust
//! const FERRIS_PNG: Asset = vg::asset!("textures/ferris.png");
//!
//! #[vg::game]
//! async fn example_game(vg: Vg) {
//!     vg.title("Example game");
//!
//!     let ferris_png = PngSource::new(FERRIS_PNG);
//!     let ferris = vg.texture(LazySource::new(Color::hex(0xFF00FF), ferris_png));
//!     
//!     loop {
//!         let trans = Matrix::identity()
//!             .scale(0.1)
//!             .rotate(vg.now());
//!
//!         vg.fill(Color::BLACK);
//!         vg.draw(&ferris, &[trans]);
//!         vg.present().await;
//!     }
//! }
//! ```

#![deny(where_clauses_object_safety)]

pub use async_trait::async_trait;

/// Typedef around [usize; 2] representing the size of something, usually in
/// pixels
pub type Size = [usize; 2];
/// A 4x4 matrix
pub type Matrix = [f32; 16];
/// A timestamp
pub type Time = f64;
/// A 2D coordinate
pub type Coord = [f32; 2];

pub const EPOCH: Time = 0.0;

mod color;
pub use color::{Color, ColorExt};
mod asset;
pub mod gfx;
mod macros;
pub mod sfx;
pub use asset::Asset;
pub mod input;

/// Main handle to vg for comfort
pub struct Vg(Box<dyn Api>);

pub trait Api: gfx::Gfx + sfx::Sfx + input::Input {
    /// Set the window title
    fn title(&mut self, title: String);
    /// Resize the window
    fn resize(&mut self, size: WindowSize);
    /// Set the window icon
    fn icon(&mut self, icon: Icon);
    /// Set fullscreen
    fn fullscreen(&mut self, fullscreen: bool);
    /// Set vsync
    fn vsync(&mut self, vsync: bool);

    /// Get the current time
    fn now(&self) -> Time;
}

/// Application window size preference
///
/// Note: Not all platforms will respect this, for example on Web resizing is
/// "best effort"
pub enum WindowSize {
    /// Lock the window size to something specific, disallowing user resize
    Fixed(Size),
    /// Make sure the window is larger than `min` and smaller than `max`
    Between { min: Size, max: Size },
    /// Make the window fullscreen
    Fullscreen,
}

pub struct Icon;
