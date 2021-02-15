use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};
use ultraviolet::Vec3;
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
    pub const HOST: PlayerId = PlayerId(0);

    pub fn from_hash(hash: impl std::hash::Hash) -> PlayerId {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hash.hash(&mut hasher);
        PlayerId(hasher.finish())
    }

    pub fn is_host(&self) -> bool {
        Self::HOST == *self
    }
}

impl Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#X}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerEvent {
    pub player: PlayerId,
    pub kind: Event,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[repr(u8)]
pub enum Event {
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

impl Event {
    pub(crate) fn new(ev: &WEvent<()>) -> Option<Event> {
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
                    return Some(Event::Key(KeyEvent {
                        key: *key,
                        state: if let ElementState::Pressed = state {
                            Digital::Pressed
                        } else {
                            Digital::Released
                        },
                    }))
                }
                WindowEvent::ReceivedCharacter(char) => return Some(Event::Text(*char)),
                _ => (),
            }
        }
        None
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InputCache {
    keys: HashMap<Key, Digital>,
}

impl InputCache {
    pub fn new() -> InputCache {
        InputCache {
            keys: HashMap::new(),
        }
    }

    pub fn event(&mut self, event: &Event) {
        match event {
            Event::Key(KeyEvent { key, state }) => {
                self.keys.insert(*key, *state);
            }
            _ => (),
        }
    }

    pub fn tick(&mut self) {
        for k in self.keys.values_mut() {
            match k {
                Digital::Pressed => *k = Digital::Down,
                Digital::Released => *k = Digital::Up,
                _ => (),
            }
        }
    }

    pub fn key(&self, key: Key) -> Digital {
        self.keys.get(&key).copied().unwrap_or_default()
    }

    /// Get the state of WASD and Arrow keys as a Vec3, so you can apply this easily a Transform.position
    pub fn wasd_arrows(&self) -> Vec3 {
        let mut res = Vec3::zero();
        if self.key(Key::W).down() || self.key(Key::Up).down() {
            res.y += 1.0;
        }
        if self.key(Key::S).down() || self.key(Key::Down).down() {
            res.y -= 1.0;
        }
        if self.key(Key::D).down() || self.key(Key::Right).down() {
            res.x += 1.0;
        }
        if self.key(Key::A).down() || self.key(Key::Left).down() {
            res.x -= 1.0;
        }
        res
    }
}
