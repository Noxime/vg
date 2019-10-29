//! # Overview
//! vg is a lightweight game engine / framework, intended to abstract all the
//! platform specific parts to building a 2D game.
//!
//! vg is currently in heavy development, and many features are still missing

use std::fmt::Display;

pub use async_trait::async_trait;

pub type Size = [usize; 2];
pub type Color = [f32; 4];
pub type Matrix = [f32; 16];
pub type Time = f64;

pub mod gfx;
mod asset;
pub use asset::Asset;

pub trait Api: gfx::Gfx {
    /// set the window title
    fn title(&mut self, title: impl Display);
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

    fn assert(&self, path: &str) -> Option<Asset>;
}

pub struct Icon;
