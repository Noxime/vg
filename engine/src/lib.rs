//! # Overview
//! vg is a lightweight game engine / framework, intended to abstract all the
//! platform specific parts to building a 2D game.
//!
//! # Example
//! ```rust
//! const FERRIS_PNG: Asset = vg::asset!("textures/ferris.png");
//!
//! #[vg::game]
//! async fn main(gfx: Gfx, sfx: Sfx, input: Input) {
//!     vg.title("Example game");
//!
//!     let ferris = gfx.texture(PngSource::new(FERRIS_PNG)).await;
//!     
//!     while input.next_event() != Some(Event::Exit) {
//!
//!         let transform = Matrix::identity()
//!             .scale(0.1)
//!             .rotate(vg.now());
//!
//!         gfx.fill(Color::BLACK);
//!         gfx.draw(&ferris, &[transform]);
//!         gfx.present().await;
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
/// A 2D coordinate
pub type Coord = [f32; 2];

mod time;
pub use time::Time;
mod color;
pub use color::{Color, ColorExt};
mod asset;
pub mod gfx;
//mod macros;
pub mod sfx;
pub use asset::Asset;
pub mod input;

pub use {gfx::Gfx, input::Input, sfx::Sfx};

pub struct Vg {
    vg: Box<dyn VgTrait>,
    input: input::Input,
}

impl Vg {
    #[cfg_attr(not(feature = "dev-docs"), doc(hidden))]
    pub fn new(vg: Box<dyn VgTrait>) -> Vg {
        Vg {
            vg,
            input: input::Input::new(),
        }
    }

    /// Set the window title
    pub fn title(&mut self, title: impl AsRef<str>) {
        self.vg.title(title.as_ref())
    }

    pub fn resize(&mut self, window: WindowMode) {
        self.vg.resize(window)
    }

    /// Get an event from the event queue, if there is any
    ///
    /// You should poll this at the beginning of every frame, to make sure you
    /// are up to date on inputs
    pub fn poll_event(&mut self) -> Option<Event> {
        if let Some(e) = self.vg.poll_event() {
            self.input.handle(self.time(), &e);
            Some(e)
        } else {
            None
        }
    }

    /// Get the current time
    pub fn time(&self) -> Time {
        self.vg.time()
    }
}

#[cfg_attr(not(feature = "dev-docs"), doc(hidden))]
pub trait VgTrait {
    /// Set the window title
    fn title(&mut self, title: &str);

    fn resize(&mut self, window: WindowMode);

    /// Get the next event from the event queue, if possible
    // TODO: Explore the possibilities of making this an async fn or a stream
    fn poll_event(&mut self) -> Option<Event>;

    /// Current time, as high resolution as possible and since the call of game
    /// entrypoint
    fn time(&self) -> Time;
}

/// Event
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// The game was requested to exit
    Exit,
    /// The game has been focused, or returned from the background on mobile
    FocusGained,
    /// The game is now in the background
    ///
    /// # Note
    /// This is sent when your game goes to background on mobile. This means
    /// you should save your app data as quickly as possible, as apps
    /// may/will get shut down in a few seconds
    FocusLost,
    /// Window changed size
    Resize(Size),
    /// Keyboard event
    Keyboard(input::keyboard::Event),
    /// Mouse event
    Mouse(input::mouse::Event),
    /// Gamepad event
    Gamepad {
        id: input::gamepad::Id,
        ev: input::gamepad::Event,
    },
    /// Touch event
    Touch {
        id: input::touch::Id,
        ev: input::touch::Event,
    },
}

/// Window configuration
///
/// Note: Not all platforms are guaranteed to respect this, for example on Web
/// fullscreen may be denied or resizing is unavailable
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WindowMode {
    Fullscreen,
    WindowedFullscreen,
    Windowed(WindowSize),
}

/// Application window size preference
///
/// Note: Not all platforms will respect this, for example on Web resizing is
/// "best effort"
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WindowSize {
    /// The window is allowed to be any size
    Any,
    /// Lock the window size to something specific, disallowing user resize
    Fixed(Size),
    /// Make sure the window is larger than `min` and smaller than `max`
    Between { min: Size, max: Size },
}

impl Default for WindowMode {
    fn default() -> Self {
        Self::Windowed(Default::default())
    }
}

impl Default for WindowSize {
    fn default() -> Self {
        Self::Any
    }
}
