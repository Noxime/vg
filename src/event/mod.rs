use serde::{Deserialize, Serialize};
use winit::event::Event as WEvent;

mod input;
use input::{Analog, Digital, GamepadEvent, KeyEvent, MouseEvent, TouchEvent};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlayerId(pub(crate) u64);

impl<H: std::hash::Hash> From<H> for PlayerId {
    fn from(hash: H) -> PlayerId {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hash.hash(&mut hasher);
        PlayerId(hasher.finish())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub player: PlayerId,
    pub kind: EventKind,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EventKind {
    /// A player has connected to this game
    Connected,
    /// A player has disconnected from this game
    Disconnected,
    /// Text input
    Text(char),
    /// Keyboard input
    Key(KeyEvent),
    /// Mouse input
    Mouse(MouseEvent),
    /// Touch input
    Touch(TouchEvent),
    /// Gamepad input
    Gamepad(GamepadEvent),
}

impl EventKind {
    pub(crate) fn new(ev: &WEvent<()>) -> Option<EventKind> {
        use winit::event::{ElementState, KeyboardInput, WindowEvent};
        if let WEvent::WindowEvent { event, .. } = ev {
            match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(key),
                            state,
                            ..
                        },
                    ..
                } => {
                    return Some(EventKind::Key(KeyEvent {
                        key: *key,
                        state: if let ElementState::Pressed = state {
                            Digital::Pressed
                        } else {
                            Digital::Released
                        },
                    }))
                }
                WindowEvent::ReceivedCharacter(char) => return Some(EventKind::Text(*char)),
                _ => (),
            }
        }
        None
    }
}
