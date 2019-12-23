mod gamepad;
pub use gamepad::{Event as GamepadEvent, Id as GamepadId};

/// An input event
pub enum Event {
    Gamepad {
        id: GamepadId,
        ev: GamepadEvent,
    },
    Keyboard,
    Mouse,
    Touch,
}

pub trait Input {
    /// Poll all the input events
    fn events(&self) -> Vec<Event>;
}