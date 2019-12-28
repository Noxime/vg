//! Gamepad API

use std::collections::HashMap;

use uuid::Uuid;

/// A unique Id for a gamepad that has been connected at one point
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Id(#[doc(hidden)] usize);

/// A gamepad generated event
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Event {
    /// A gamepad has been connected. If this gamepad has been connected
    /// before, the ID is re-used. Otherwise a new ID is assigned
    Connected,
    /// A controller has been disconnected
    Disconnected,
    /// The value of a button has changed
    ///
    /// If you want a binary state of a button, see [`Event::ButtonState`]
    ButtonValue(Button, f32),
    /// The value of a joystick axis has changed
    ///
    /// Ranges from -1.0 to 1.0, with 0.0 being the center
    AxisValue(Axis, f32),
    // special
    /// A digital button state change. True if pressed, false if released
    ///
    /// Automatically generated from [`Event::ButtonState`] events
    ButtonState(Button, bool),
}

/// Information about a gamepad that may or may not be connected currently, but
/// has been connected at one point
#[derive(Clone, Debug, PartialEq)]
pub struct Info {
    /// The uuid of this controller
    pub uuid: Uuid,
    /// Is the gamepad connected currently
    pub connected: bool,
    /// A user friendly name for the gamepad
    pub name: String,
    /// Current gamepad power information
    pub power: Power,
    /// Does this controller support force-feedback
    pub force_feedback: bool,
}

/// The current power state of a gamepad
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Power {
    Unknown,
    Wired,
    /// Battery percentage in range `0.0` - `1.0`
    Discharging(f32),
    /// Battery percentage in range `0.0` - `1.0`
    Charging(f32),
    /// Fully charged
    Charged,
}

/// A digital button on a gamepad
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Button {
    Select,
    Start,
    /// The "logo" button
    Mode,

    DPadUp,
    DPadRight,
    DPadDown,
    DPadLeft,

    /// Face button thats on top. On Xbox "Y", on Switch "X"
    FaceUp,
    /// Face button thats on top. On Xbox "B", on Switch "A"
    FaceRight,
    /// Face button thats on top. On Xbox "A", on Switch "B"
    FaceDown,
    /// Face button thats on top. On Xbox "X", on Switch "Y"
    FaceLeft,

    LeftBumper,
    RightBumper,
    LeftTrigger,
    RightTrigger,

    LeftThumb,
    RightThumb,
}

/// An analog axis on a gamepad, such as a joystick
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    /// Left joystick X
    LeftX,
    /// Left joystick Y
    LeftY,

    /// Right joystick X
    RightX,
    /// Right joystick Y
    RightY,

    /// Left trigger
    LeftZ,
    /// Right trigger
    RightZ,

    DPadX,
    DPadY,
}

/// A simple, gamepad state for convenience
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    buttons: HashMap<Button, Digital>,
    axes: HashMap<Axis, Analog>,
}

impl State {
    /// Get the state of a button on the controller
    pub fn button(&self, button: Button) -> Digital {
        self.buttons.get(&button).cloned().unwrap_or_default()
    }
    /// Get the state of an axis on the controller
    pub fn axis(&self, axis: Axis) -> Analog {
        self.axes.get(&axis).cloned().unwrap_or_default()
    }
}

/// A digital state of a button on a gamepad
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Digital {
    /// The input was released on this frame
    Released,
    /// The input is idle
    Up,
    /// The input has been pressed on this frame
    Pressed,
    /// The button is currently down
    Held,
}

impl Digital {
    /// A convenience function that returns `true` if `self` is
    /// [`Digital::Pressed`] or [`Digital::Held`]
    pub fn down(&self) -> bool {
        match self {
            Digital::Pressed | Digital::Held => true,
            _ => false,
        }
    }

    /// A convenience function that returns `true` if the input is considered to
    /// be lifted
    pub fn up(&self) -> bool {
        !self.down()
    }
}

/// An analog state of an axis on a gamepad
#[derive(Copy, Clone, Debug, PartialEq)]
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
}

impl core::ops::Deref for Analog {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        // deadzone
        if self.0.abs() < 0.05 {
            return &0.0;
        }
        &(self.0)
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
