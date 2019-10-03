//! # Overview
//! Kea is a lightweight game engine / framework, intended to abstract all the
//! platform specific parts to building a 2D game.
//!
//! Kea is currently in heavy development, and many features are still missing
//! Check the [issues](https://owo.codes/noxim/kea/issues) to see how everything
//! is coming along :)
//!
//! # Getting started
//! ```rust
//! use kea::{Game, Entrypoint};
//! 
//! struct Example;
//! impl Game for Example {
//!     // The type signature for this function is bit dumb,
//!     // but it is to allow the game to pass the FFI
//!     fn new(config: &mut Config) -> Box<Box<dyn Game>> {
//!         config.title("Ferris 2: Kona");
//!         config.size([800, 600]);
//!         Box::new(Box::new(Example))
//!     }
//! 
//!     // This is your main loop
//!     fn update(&mut self, api: &mut Api) {
//!         api.fill(Color::BLUE);
//!     }
//! }
//! 
//! // This is how you actually tell Kea what your game is
//! pub const KEA_GAME: Entrypoint = &Example::new;
//! ```
//!
//! Noxim, 2019-10-03

pub mod audio;
pub mod input;
pub mod renderer;

#[macro_use]
pub mod assets;

pub use self::audio::Audio;
pub use self::input::Input;
pub use self::renderer::Renderer;

pub type Size = [usize; 2];
pub type Color = [f32; 4];
pub type Transform = [f32; 4*4];
pub type Entrypoint = *const dyn Fn(&mut Config) -> Box<Box<dyn Game>>;

pub trait Game {
    fn new(config: &mut Config) -> Box<Box<dyn Game>>;
    fn update(&mut self, api: &mut Api);
    fn resume(&mut self, api: &mut Api) { api.config().update(Update::Always); }
    fn suspend(&mut self, api: &mut Api) { api.config().update(Update::Never); }
}

/// An `Api` is your handle to the game engine
pub struct Api {
    config: Config,
}

impl Api {
    pub fn controller(&self) -> Option<input::Controller> {
        unimplemented!()
    }

    pub fn controllers(&self) -> Vec<input::Controller> {
        unimplemented!()
    }

    pub fn pointer(&self) -> Option<input::Pointer> {
        unimplemented!()
    }

    pub fn pointers(&self) -> Vec<input::Pointer> {
        unimplemented!()
    }

    /// Configure Kea behavior
    pub fn config(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Access to static assets such as textures and sounds
    pub fn assets(&self) -> assets::Assets {
        unimplemented!()
    }

    /// Create a sound clip from an audio source
    /// 
    /// # Example
    /// ```rust
    /// let bytes = api.assets()["sounds/cat.ogg"]?;
    /// let sound = api.sound(ogg(bytes)?);
    /// ```
    pub fn sound(&self, source: impl audio::Source) -> audio::Sound {
        unimplemented!()
    }
    
    /// Create a texture from a decoder
    /// 
    /// # Example
    /// ```rust
    /// let bytes = api.assets()["textures/lena.png"]?;
    /// let tex = api.texture(png(bytes)?);
    /// ```
    pub fn texture(&self, data: impl TextureSource) -> Texture {
        unimplemented!()
    }
}

/// Configure how Kea behaves
pub struct Config {
    /// Window / application / tab title
    title: String,
    /// The icon / favicon
    icon: Option<Icon>,
    /// Window size
    size: Size,
    /// How should window resizing be handled
    resize: Resize,
    /// Is fullscreen?
    fullscreen: bool,
    /// Should the main loop run
    update: Update,
}

impl Config {
    pub fn title(&mut self, title: impl std::fmt::Display) {
        self.title = format!("{}", title);
    }

    pub fn icon(&mut self, icon: Option<Icon>) {
        self.icon = icon;
    }

    pub fn size(&mut self, size: impl Into<Size>) {
        self.size = size.into();
    }

    pub fn resize(&mut self, resize: impl Into<Resize>) {
        self.resize = resize.into();
    }

    pub fn fullscreen(&mut self, fullscreen: bool) {
        self.fullscreen = fullscreen;
    }

    pub fn update(&mut self, update: Update) {
        self.update = update;
    }
}

/// How to handle window resizing
pub enum Resize {
    /// Allow full window resizing (default)
    Allow,
    /// Deny all window resizes
    Deny,
    /// Keep the window bigger or same (component wise) as this size
    /// 
    /// Note, on some platforms such as the web this cannot be guaranteed
    Min(Size),
}

impl Default for Resize {
    fn default() -> Resize {
        Resize::Allow
    }
}

/// How should the main loop be run
pub enum Update {
    /// As fast as possible (default)
    Always,
    /// Don't run
    Never,
    /// Run based on monitor frequency
    Vsync,
}

impl Default for Update {
    fn default() -> Update {
        Update::Always
    }
}

pub struct Icon;