use crate::{input, Size};

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// The game was requested to exit
    ///
    /// This event _must_ be handled, and if game does not willingly quit, it
    /// will be shut down
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
