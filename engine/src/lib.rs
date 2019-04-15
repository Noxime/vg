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
//! The way Kea is structured is little annoying to use, but allows us to write
//! platform specific code very cleanly in their own crates. Structuring kea
//! "traditionally" would make building it quite difficult for targets like iOS
//! or the Nintendo Switch
//! 
//! ## Steps
//! Start by cloning the repository somewhere on your disk
//! ```bash
//! $ git clone https://owo.codes/noxim/kea && cd kea
//! ```
//! 
//! Create your game crate, it should be a library since targets depend on it
//! ```bash
//! $ cargo new tetris --lib
//! ```
//! 
//! You have to configure kea for it to know where your game is, so run
//! ```bash
//! $ ./configure tetris
//! ```
//! 
//! Now, your game needs an entry point. Go to `tetris/src/lib.rs` and change
//! it to 
//! ```rust
//! use kea::*;
//! 
//! pub fn game<P, R, I>(mut api: EngineApi<P, R, I>)
//! where
//!     P: PlatformApi,
//!     R: renderer::Renderer,
//!     I: input::Input,
//! {
//!     // This is your main function
//! }
//! ```
//! 
//! Also remember to add kea to your dependencies in `tetris/Cargo.toml` :)
//! ```toml
//! [dependencies]
//! kea = { path = "../engine" }
//! ```
//! 
//! Now you should be all set to build and run! Although, your game does nothing
//! and will immediately exit :P
//! ```bash
//! $ cd target-desktop && cargo run
//! ```
//! 
//! ---
//! Noxim, 2019-04-15

pub mod platform_api;
pub mod renderer;
pub mod input;
pub mod audio;
// TODO: Move the macros back inside `assets`
#[macro_use]
pub mod assets;
pub use self::platform_api::PlatformApi;
pub use self::renderer::Renderer;
pub use self::input::Input;

pub struct EngineApi<Platform: PlatformApi, Renderer: renderer::Renderer, Input: input::Input> {
    pub platform: Platform,
    pub renderer: Renderer,
    pub input: Input,
    pub poll: Box<FnMut()>,
}

impl<Platform: PlatformApi, Renderer: renderer::Renderer, Input: input::Input> EngineApi<Platform, Renderer, Input> {
    pub fn poll(&mut self) {
        (self.poll)()
    }
}

pub fn run<Platform: PlatformApi, Renderer: renderer::Renderer, Input: input::Input>(
    platform: Platform,
    renderer: Renderer,
    input: Input,
    poll: Box<FnMut()>,
    game: &Fn(EngineApi<Platform, Renderer, Input>),
) {
    let engine = EngineApi {
        platform,
        renderer,
        input,
        poll,
    };
    engine.platform.print("Running Kea");
    game(engine);
}
