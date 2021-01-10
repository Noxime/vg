use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};
use winit::event::Event as WEvent;

mod input;
pub use input::{Analog, Digital, GamepadEvent, Key, KeyEvent, MouseEvent, TouchEvent};

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct PlayerId(pub(crate) u64);

// impl<H: std::hash::Hash> From<H> for PlayerId {
//     fn from(hash: H) -> PlayerId {
//         use std::hash::Hasher;
//         let mut hasher = std::collections::hash_map::DefaultHasher::new();
//         hash.hash(&mut hasher);
//         PlayerId(hasher.finish())
//     }
// }

impl PlayerId {
    pub const ALL: PlayerId = PlayerId(0);
    pub fn from_hash(hash: impl std::hash::Hash) -> PlayerId {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hash.hash(&mut hasher);
        PlayerId(hasher.finish())
    }
}

impl Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#X}", self.0)
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

#[derive(Clone)]
pub struct InputCache {
    keys: HashMap<PlayerId, HashMap<Key, Digital>>,
}

impl InputCache {
    pub fn new() -> InputCache {
        InputCache {
            keys: HashMap::new(),
        }
    }

    pub fn event(&mut self, event: &Event) {
        match event.kind {
            EventKind::Key(KeyEvent { key, state }) => {
                self.keys
                    .entry(event.player)
                    .or_default()
                    .insert(key, state);
                self.keys
                    .entry(PlayerId::ALL)
                    .or_default()
                    .insert(key, state);
            }
            _ => (),
        }
    }

    pub fn tick(&mut self) {
        for (_, m) in self.keys.iter_mut() {
            for k in m.values_mut() {
                match k {
                    Digital::Pressed => *k = Digital::Down,
                    Digital::Released => *k = Digital::Up,
                    _ => (),
                }
            }
        }
    }

    pub fn key(&self, player: PlayerId, key: Key) -> Digital {
        self.keys
            .get(&player)
            .unwrap_or(&HashMap::new())
            .get(&key)
            .copied()
            .unwrap_or_default()
    }
}
