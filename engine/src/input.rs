use crate::Coord;

pub mod gamepad;
use gamepad::{
    Event as GamepadEvent, Id as GamepadId, Info as GamepadInfo,
    State as GamepadState,
    Digital,
};

pub mod keyboard;
use keyboard::{Event as KeyboardEvent, VirtualKey};

pub mod mouse;
use mouse::{Event as MouseEvent, Button as MouseButton};

pub mod touch;
use touch::{Event as TouchEvent, Id as TouchId};

/// An input event
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Event {
    /// Event coming from a gamepad, identified by its `id`
    Gamepad {
        id: GamepadId,
        ev: GamepadEvent,
    },
    Keyboard {
        ev: KeyboardEvent,
    },
    Mouse {
        ev: MouseEvent,
    },
    Touch {
        id: TouchId,
        ev: TouchEvent,
    },
}

/// The input API
pub trait Input {
    /// Poll all the input events
    fn events(&self) -> Vec<Event>;
    /// Get information about a gamepad, or `None` if ID is not known / valid
    fn info(&self, id: GamepadId) -> Option<GamepadInfo>;
    /// Get the state of a gamepad based on its ID
    fn gamepad(&self, id: GamepadId) -> Option<GamepadState>;
    /// Get the state of a key
    fn key(&self, key: VirtualKey) -> Digital;
    /// Get the position of the mouse
    fn mouse_position(&self) -> Coord;
    /// Get the state of a mouse button
    fn mouse_button(&self, button: MouseButton) -> Digital;
}
