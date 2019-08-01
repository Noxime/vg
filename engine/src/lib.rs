//! # Overview
//! Kea is a lightweight game engine / framework, intended to abstract all the
//! platform specific parts to building a 2D game. As of now
//! [(2019-04-13)](https://owo.codes/noxim/kea/tree/28726ff2652edce8e09f886ce20ef8945d15ece9)
//! kea doesn't provide much functionality, but the basics are there.
//!
//! Kea is currently in heavy development, and many features are still missing
//! Check the [issues](https://owo.codes/noxim/kea/issues) to see how everything
//! is coming along :)
//!
//! The most important points of abstraction are
//! * [`renderer`]
//! * [`input`]
//! * [`assets`]
//! * [`audio`]
//!
//! # Getting started
//! **TODO**
//!
//! Noxim, 2019-07-28

pub mod audio;
pub mod input;
pub mod renderer;

#[macro_use]
pub mod assets;

pub use self::audio::Audio;
pub use self::input::Input;
pub use self::renderer::Renderer;

pub trait Api {
    type R: Renderer;
    type I: Input;
    type A: Audio;

    /// Run internal Kea systems, often things like input updates
    ///
    /// You should call this often, for example on every frame
    fn poll(&mut self);
    /// Does the engine want to exit, e.g. has the X been clicked
    ///
    /// You should check this in your game loop
    fn exit(&self) -> bool;
    /// Get a handle to the renderer api
    fn renderer<'a>(&'a mut self) -> &'a mut Self::R;
    /// Get a handle to the input api
    fn input<'a>(&'a mut self) -> &'a mut Self::I;
    /// Get a handle to the audio api
    fn audio<'a>(&'a mut self) -> &'a mut Self::A;
}
