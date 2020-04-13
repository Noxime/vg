//! Gamepad API

use super::{Digital, Analog};

/// A unique Id for a gamepad that has been connected at one point
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Id(pub(crate) usize);

/// A gamepad generated event
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// A gamepad has been connected. If this gamepad has been connected
    /// before, the ID is re-used. Otherwise a new ID is assigned
    Connected {
        name: String,
        force_feedback: bool,
    },
    /// A controller has been disconnected
    Disconnected,
    /// The power level of this controller changed
    Power(Power),
    /// The value of a joystick axis has changed
    ///
    /// Ranges from -1.0 to 1.0, with 0.0 being the center
    Axis(Axis, Analog),
    // special
    /// A digital button state change
    Button(Button, Digital),
}

/// Information about a gamepad that may or may not be connected currently, but
/// has been connected at one point
#[derive(Clone, Debug, PartialEq)]
pub struct Info {
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
#[repr(u8)]
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
#[repr(u8)]
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
// TODO: Maybe find some macro to not hard-code the number of buttons here
#[derive(Default, Clone, Debug, PartialEq)]
pub struct State {
    pub(crate) buttons: [Digital; 17],
    pub(crate) axes: [Analog; 8],
}

impl State {
    /// Get the state of a button on the controller
    pub fn button(&self, button: Button) -> Digital {
        self.buttons[button as usize]
    }
    /// Get the state of an axis on the controller
    pub fn axis(&self, axis: Axis) -> Analog {
        self.axes[axis as usize]
    }
}