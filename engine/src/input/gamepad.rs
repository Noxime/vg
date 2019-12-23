#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Id(#[doc(hidden)] usize);

pub enum Event {
    /// A gamepad has been connected. If this gamepad has been connected before,
    /// the ID is re-used. Otherwise a new ID is assigned
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

pub enum Button {
    Select,
    Start,
    /// The "logo" button
    Super,

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