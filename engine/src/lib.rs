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

#![deny(where_clauses_object_safety)]

pub use async_trait::async_trait;

use std::fmt::Display;

pub type Size = [usize; 2];
pub type Color = [f32; 4];
pub type Matrix = [f32; 16];
pub type Time = f64;

pub const EPOCH: Time = 0.0;

mod asset;
pub mod gfx;
mod macros;
pub mod sfx;
pub use asset::Asset;

/// Main handle to vg for comfort
pub struct Vg(Box<dyn Api>);

pub trait Api: gfx::Gfx + sfx::Sfx {
    /// Set the window title
    fn set_title(&mut self, title: String);
    /// Resize the window
    fn set_size(&mut self, size: WindowSize);
    /// Set the window icon
    fn set_icon(&mut self, icon: Icon);
    /// Set fullscreen
    fn set_fullscreen(&mut self, fullscreen: bool);
    /// Set vsync
    fn set_vsync(&mut self, vsync: bool);

    /// Get the current time
    fn now(&self) -> Time;

    /// load an asset from a path
    fn asset(&self, path: &str) -> Option<Asset>;
}

impl Vg {
    //! Configuration

    pub fn title(&mut self, title: impl Display) {
        self.0.set_title(format!("{}", title))
    }

    pub fn resize(&mut self, size: WindowSize) {
        self.0.set_size(size)
    }

    pub fn icon(&mut self, icon: Icon) {
        self.0.set_icon(icon)
    }

    pub fn vsync(&mut self, vsync: bool) {
        self.0.set_vsync(vsync)
    }

    /// Return a time that represents the current timestamp
    pub fn now(&self) -> Time {
        self.0.now()
    }

    /// Load an asset
    pub fn asset(&self, path: impl AsRef<str>) -> Option<Asset> {
        self.0.asset(path.as_ref())
    }
}

/// Application window size preference
///
/// NOTE: Not all platforms will respect this, for example on Web resizing is
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
