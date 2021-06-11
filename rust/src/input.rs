use std::collections::HashMap;

pub use vg_types::{Digital, Key};

use crate::ensure;

#[derive(Default)]
pub struct Input {
    keys: HashMap<Key, Digital>,
}

impl Input {
    pub fn set(&mut self, key: Key, state: Digital) {
        self.keys.insert(key, state);
    }

    pub fn step_states(&mut self) {
        for state in self.keys.values_mut() {
            match state {
                Digital::Raised => *state = Digital::Up,
                Digital::Pressed => *state = Digital::Down,
                _ => (),
            }
        }
    }
}

pub trait KeyExt: Sized {
    fn state(self) -> Digital;

    fn down(self) -> bool {
        *self.state()
    }

    fn up(self) -> bool {
        !self.down()
    }

    fn changed(self) -> bool {
        matches!(self.state(), Digital::Pressed | Digital::Raised)
    }

    fn pressed(self) -> bool {
        matches!(self.state(), Digital::Pressed)
    }

    fn raised(self) -> bool {
        matches!(self.state(), Digital::Raised)
    }
}

impl KeyExt for Key {
    fn state(self) -> Digital {
        key(self)
    }
}

pub fn key(key: Key) -> Digital {
    ensure()
        .input
        .keys
        .get(&key)
        .copied()
        .unwrap_or(Digital::Up)
}

pub fn wasd() -> [f32; 2] {
    fn f(k: Key) -> f32 {
        if *key(k) {
            1.0
        } else {
            0.0
        }
    }

    [f(Key::D) - f(Key::A), f(Key::W) - f(Key::S)]
}
