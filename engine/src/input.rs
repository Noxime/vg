/// A structure representing the state of a game controller
#[derive(Debug, Default, Clone)]
pub struct Controller {
    pub info: Info,
    pub start: Axis,
    pub select: Axis,
    pub left_joy: Joy,
    pub right_joy: Joy,
    pub dpad: Buttons,
    pub buttons: Buttons,
    pub left_shoulder: Shoulder,
    pub right_shoulder: Shoulder,
}

/// Generic metadata about a game controller
#[derive(Debug, Clone)]
pub struct Info {
    /// User friendly name for the game controller
    pub name: String,
    /// The current power status of the controller
    pub power: Power,
    /// ID which you can use to access a specific gamepad
    pub id: Id,
    /// Is the controller currently connected
    pub connected: bool,
}

impl Default for Info {
    fn default() -> Self {
        Info {
            name: "INTERNAL ERROR".into(),
            power: Power::Unknown,
            id: !0 - 1,
            connected: false,
        }
    }
}

/// An ID refererring to a controller
pub type Id = usize;

/// The current power state of a controller
///
/// If the variant is either `Charging(_)` or `Discharging(_)`, then the value
/// is the battery percentage from `0.0` to `1.0`
#[derive(Debug, Clone)]
pub enum Power {
    Unknown,
    Wired,
    Discharging(f32),
    Charging(f32),
    Charged,
}

/// An analog stick where each axis is in range `-1.0` to `1.0`
#[derive(Debug, Default, Clone)]
pub struct Joy {
    pub x: f32,
    pub y: f32,
}

/// A 4-way button arrangement, such as the face buttons or the D-pad
#[derive(Debug, Default, Clone)]
pub struct Buttons {
    pub up: Axis,
    pub down: Axis,
    pub left: Axis,
    pub right: Axis,
}

/// Triggers and bumpers of the controller
#[derive(Debug, Default, Clone)]
pub struct Shoulder {
    pub trigger: Axis,
    pub bumper: Axis,
}

/// A digital or analog button
#[derive(Debug, Clone)]
pub struct Axis {
    raw: f32,
    treshold: f32,
    kind: AxisKind,
}

impl Default for Axis {
    fn default() -> Self {
        Axis {
            raw: 0.0,
            treshold: 0.25,
            kind: AxisKind::Digital,
        }
    }
}

/// Is axis a digital or analog input
///
/// Note: You generally do not need to use this, as [`Axis`] abstracts the
/// input type away.
#[derive(Debug, Clone)]
pub enum AxisKind {
    Digital,
    Analog,
}

impl Axis {
    pub fn active(&self) -> bool {
        self.raw > self.treshold
    }

    pub fn value(&self) -> f32 {
        self.raw
    }
}

impl From<bool> for Axis {
    fn from(v: bool) -> Axis {
        Axis {
            raw: if v { 1.0 } else { 0.0 },
            treshold: 0.25,
            kind: AxisKind::Digital,
        }
    }
}

impl From<f32> for Axis {
    fn from(v: f32) -> Axis {
        Axis {
            raw: v,
            treshold: 0.25,
            kind: AxisKind::Analog,
        }
    }
}

/// A mapping between a keyboard input device and a virtual [`Controller`]
/// 
/// Note: For its usage, see [`mapping`](Input::mapping) and 
/// [`set_mapping`](Input::set_mapping)
/// 
/// The values for a [`default`](Default::default) mapping are:
/// 
/// | Controller     | Keyboard                                     |
/// |----------------|----------------------------------------------|
/// | Start          | Escape                                       |
/// | Select         | Enter                                        |
/// | Left joystick  | WASD                                         |
/// | Right joystick | IJKL                                         |
/// | DPAD           | Arrow keys                                   |
/// | Buttons        | Down is Q, right is E, left is U and up is O |
/// | Triggets       | Shifts                                       |
/// | Bumpers        | Alt, Control and Super keys                  |
#[derive(Debug)]
pub struct KeyboardMapping {
    pub start: Vec<Key>,
    pub select: Vec<Key>,

    pub left_joy_up: Vec<Key>,
    pub left_joy_down: Vec<Key>,
    pub left_joy_left: Vec<Key>,
    pub left_joy_right: Vec<Key>,

    pub right_joy_up: Vec<Key>,
    pub right_joy_down: Vec<Key>,
    pub right_joy_left: Vec<Key>,
    pub right_joy_right: Vec<Key>,

    pub dpad_up: Vec<Key>,
    pub dpad_down: Vec<Key>,
    pub dpad_left: Vec<Key>,
    pub dpad_right: Vec<Key>,

    pub buttons_up: Vec<Key>,
    pub buttons_down: Vec<Key>,
    pub buttons_left: Vec<Key>,
    pub buttons_right: Vec<Key>,

    pub left_shoulder_trigger: Vec<Key>,
    pub left_shoulder_bumper: Vec<Key>,
    pub right_shoulder_trigger: Vec<Key>,
    pub right_shoulder_bumper: Vec<Key>,
}

impl Default for KeyboardMapping {
    fn default() -> KeyboardMapping {
        KeyboardMapping {
            start: vec![Key::Esc],
            select: vec![Key::Enter],

            left_joy_up: vec![Key::W],
            left_joy_down: vec![Key::S],
            left_joy_left: vec![Key::A],
            left_joy_right: vec![Key::D],

            right_joy_up: vec![Key::I],
            right_joy_down: vec![Key::K],
            right_joy_left: vec![Key::J],
            right_joy_right: vec![Key::L],

            dpad_up: vec![Key::Up],
            dpad_down: vec![Key::Down],
            dpad_left: vec![Key::Left],
            dpad_right: vec![Key::Right],

            buttons_up: vec![Key::O],
            buttons_down: vec![Key::Q],
            buttons_left: vec![Key::U],
            buttons_right: vec![Key::E],

            left_shoulder_trigger: vec![Key::LShift],
            left_shoulder_bumper: vec![Key::LAlt, Key::LCtrl, Key::LSuper],
            right_shoulder_trigger: vec![Key::RShift],
            right_shoulder_bumper: vec![Key::RAlt, Key::RCtrl, Key::RSuper],
        }
    }
}

/// A key on the keyboard, used for [`KeyboardMapping`]
/// 
/// Note: This enum does not contain all the keys present on keyboards, but
/// rather a subset that I personally see useful for games.
#[derive(Debug)]
pub enum Key {
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    /// Escape key
    Esc,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    /// Arrow key up
    Up,
    /// Arrow key down
    Down,
    /// Arrow key left
    Left,
    /// Arrow key right
    Right,

    /// Backspace key
    Back,
    /// Enter key
    Enter,
    /// Spacebar
    Space,

    Period,
    Comma,
    Minus,

    /// Left shift modifier
    LShift,
    /// Left alt modifier
    LAlt,
    /// Left ctrl modifier
    LCtrl,
    /// Left "super" key, on windows this is the Winkey, on MacOS it is "cmd"
    /// and so on.
    LSuper, 

    /// Right shift modifier
    RShift,
    /// Right alt modifier
    /// 
    /// Note: Avoid using this without an alterantive set, as some keyboards do
    /// not have this key
    RAlt,
    /// Right ctrl modifier
    /// 
    /// Note: Avoid using this without an alterantive set, as some keyboards do
    /// not have this key
    RCtrl,
    /// Right "super" key, on windows this is the Winkey, on MacOS it is "cmd" 
    /// and so on.
    /// 
    /// Note: Avoid using this without an alterantive set, as some keyboards do
    /// not have this key
    RSuper, 
}

impl Key {
    /// Get all modifier keys for the left side of the keyboard
    /// 
    /// This is [`LShift`](Key::LShift), [`LAlt`](Key::LAlt),
    /// [`LCtrl`](Key::LCtrl) and [`LSuper`](Key::LSuper)
    pub fn left_mods() -> Vec<Key> {
        vec![Key::LShift, Key::LAlt, Key::LCtrl, Key::LSuper]
    }

    /// Get all modifier keys for the right side of the keyboard
    /// 
    /// This is [`RShift`](Key::RShift), [`RAlt`](Key::RAlt), 
    /// [`RCtrl`](Key::RCtrl) and [`RSuper`](Key::RSuper)
    pub fn righs_mods() -> Vec<Key> {
        vec![Key::RShift, Key::RAlt, Key::RCtrl, Key::RSuper]
    }

    /// Get both shift modifier keys
    pub fn shifts() -> Vec<Key> {
        vec![Key::LShift, Key::RShift]
    }

    /// Get both alt modifier keys
    pub fn alts() -> Vec<Key> {
        vec![Key::LAlt, Key::RAlt]
    }

    /// Get both ctrl modifier keys
    pub fn ctrls() -> Vec<Key> {
        vec![Key::LCtrl, Key::RCtrl]
    }

    /// Get both super modifier keys
    pub fn supers() -> Vec<Key> {
        vec![Key::LSuper, Key::RSuper]
    }
}

/// A [`Pointer`] represents either a mouse cursor or a touch on a screen or
/// touchpad
#[derive(Debug)]
pub struct Pointer {
    /// The [`Id`] of this pointer, which can be used to keep track of multiple
    /// fingers on a touch screen.
    pub id: Id,
    /// Normalized X position of the pointer, with `-1.0` being the left edge
    /// of the screen and `1.0` being the right edge of the screen
    pub x: f32,
    /// Normalized Y position of the pointer, with `-1.0` being the bottom edge
    /// of the screen and `1.0` being the top edge of the screen
    pub y: f32,
    /// Is the pointer currently pressed down
    /// 
    /// Note: On devices that support touch pressure, this will analog value,
    /// and on others such as the mouse this will be `0.0` or `1.0`
    pub pressed: Axis,
}

/// A cross platform input api
/// 
/// This trait provides an abstracted way of dealing with game input. There are
/// 2 forms of input in kea, a [`Controller`] and a [`Pointer`]
/// 
/// # Controllers
/// [`Controller`] represents a generic game controller, which will likely be
/// your primary input method. Keyboard events are mapped to a [`Controller`]
/// too, and how that is done can be configured with [`Input::set_mapping`]
/// 
/// # Pointers
/// [`Pointer`] represents any pointer, like the mouse cursor or touch screen
/// presses. See its documentation for more info
/// 
pub trait Input {
    /// Get the [`Id`] for the "primary" controller. You should use this for
    /// single player games, and should be the controller that most recently
    /// received input
    fn default(&self) -> Option<Id>;
    /// Get the [`Id`]s for all currently connected controllers
    ///
    /// Note: To also get controllers that have been disconnected, use
    /// [`all_controllers`](Input::all_controllers)
    fn controllers(&self) -> Vec<Id> {
        self.all_controllers()
            .into_iter()
            .filter(|id| self.controller(id).map(|c| c.info.connected)
                .unwrap_or(false))
            .collect()
    }
    /// Get the [`Id`]s for all controllers
    ///
    /// Note: This also contains controllers that are currently disconnected
    fn all_controllers(&self) -> Vec<Id>;
    /// Retrieve a controller by its [`Id`], or `None` if it does not exist
    fn controller(&self, id: &Id) -> Option<Controller>;
    /// Get pointers that we know of
    /// 
    /// To keep track of multiple touches, use the [`id`](Pointer::id) field of
    /// [`Pointer`]
    /// 
    /// Note: On PS4 Dualshock contollers the [`id`](Pointer::id) of a 
    /// [`Pointer`] is the same as the [`id`](Info::id) of the
    /// controller. This currently limits the number of touches on a Dualshock
    /// controller to only one, but hopefully later I will figure out a nice
    /// way to handle that :)
    fn pointers(&self) -> Vec<Pointer>;
    /// Get the current keyboard to [`Controller`] mapping
    /// 
    /// Note: To modify the current mapping, see 
    /// [`set_mapping`](Input::set_mapping)
    fn mapping(&self) -> KeyboardMapping;
    /// Set the current keyboard to [`Controller`] mapping
    /// 
    /// Note: To get the current mapping, see [`mapping`](Input::mapping)
    /// 
    /// Kea maps all keyboard events to a virtual game controller, and this
    /// mapping is done through a [`KeyboardMapping`]. See its documentation
    /// for the default values.
    fn set_mapping(&mut self, mapping: KeyboardMapping);
}
