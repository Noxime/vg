use serde::{Deserialize, Serialize};

const DEADZONE: f32 = 0.05;

pub use winit::event::VirtualKeyCode as Key;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyEvent {
    pub key: Key,
    pub state: Digital,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GamepadEvent {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MouseEvent {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TouchEvent {}

/// A digital state of a button or a key
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
#[repr(u8)]
pub enum Digital {
    /// The input was released on this frame
    Released,
    /// The input is idle
    Up,
    /// The input has been pressed on this frame
    Pressed,
    /// The button is currently down
    Down,
}

impl Digital {
    /// A convenience function that returns `true` if `self` is
    /// [`Digital::Pressed`] or [`Digital::Down`]
    pub fn down(&self) -> bool {
        match self {
            Digital::Pressed | Digital::Down => true,
            _ => false,
        }
    }

    /// A convenience function that returns `true` if the input is considered to
    /// be lifted
    pub fn up(&self) -> bool {
        !self.down()
    }

    /// Get this digital input as a simulated analog input
    pub fn analog(&self) -> Analog {
        (*self).into()
    }
}

/// An analog input, like the joysticks of a gamepad
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct Analog(f32);

impl Analog {
    /// Get the current value with a small dead zone applied to prevent stick
    /// drift. Use `self.raw()` if you want the raw value instead
    pub fn value(&self) -> f32 {
        **self
    }

    /// Get the raw joystick value with no deadzone and calibration added
    pub fn raw(&self) -> f32 {
        self.0
    }

    /// Get this analog input as a simulated digital input
    pub fn digital(&self) -> Digital {
        (*self).into()
    }
}

impl core::ops::Deref for Analog {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        if self.0.abs() <= DEADZONE {
            return &0.0;
        }
        &self.0
    }
}

impl Default for Digital {
    fn default() -> Self {
        Digital::Up
    }
}

impl Default for Analog {
    fn default() -> Self {
        Analog(0.0)
    }
}

impl From<Analog> for Digital {
    fn from(a: Analog) -> Digital {
        if a.abs() > DEADZONE {
            Digital::Down
        } else {
            Digital::Up
        }
    }
}

impl From<Digital> for Analog {
    fn from(d: Digital) -> Analog {
        if d.down() {
            Analog(1.0)
        } else {
            Analog(0.0)
        }
    }
}
