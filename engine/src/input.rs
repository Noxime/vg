pub mod gamepad;
use gamepad::{
    Event as GamepadEvent, Id as GamepadId, Info as GamepadInfo,
    State as GamepadState,
};

/// An input event
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Event {
    /// Event coming from a gamepad, identified by its `id`
    Gamepad { id: GamepadId, ev: GamepadEvent },
    Keyboard,
    Mouse,
    Touch,
}

/// The input API
pub trait Input {
    /// Poll all the input events
    fn events(&self) -> Vec<Event>;
    /// Get information about a gamepad, or `None` if ID is not known / valid
    fn info(&self, id: GamepadId) -> Option<GamepadInfo>;
    /// Get the state of a gamepad based on its ID
    fn gamepad(&self, id: GamepadId) -> Option<GamepadState>;
}
