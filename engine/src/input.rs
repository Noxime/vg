/// A structure representing the state of a game controller
#[derive(Debug)]
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
#[derive(Debug)]
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

/// An ID refererring to a controller
pub type Id = usize;

/// The current power state of a controller
///
/// If the variant is either `Charging(_)` or `Discharging(_)`, then the value
/// is the battery percentage from `0.0` to `1.0`
#[derive(Debug)]
pub enum Power {
    Unknown,
    Wired,
    Discharging(f32),
    Charging(f32),
    Charged,
}

/// An analog stick where each axis is in range `-1.0` to `1.0`
#[derive(Debug)]
pub struct Joy {
    pub x: f32,
    pub y: f32,
}

/// A 4-way button arrangement, such as the face buttons or the D-pad
#[derive(Debug)]
pub struct Buttons {
    pub up: Axis,
    pub down: Axis,
    pub left: Axis,
    pub right: Axis,
}

/// Triggers and bumpers of the controller
#[derive(Debug)]
pub struct Shoulder {
    pub trigger: Axis,
    pub bumper: Axis,
}

/// A digital or analog button
#[derive(Debug)]
pub struct Axis {
    raw: f32,
    treshold: f32,
    kind: AxisKind,
}

/// Is axis a digital or analog input
///
/// Note: You generally do not need to use this, as [`Axis`] abstracts the input
/// type away.
#[derive(Debug)]
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

/// A cross platform input api
///
///
pub trait Input {
    /// Get the [`ID`] for the "primary" controller. You should use this for
    /// single player games, and should be the controller that most recently
    /// received input
    fn default(&self) -> Option<Id>;
    /// Get the [`ID`]s for all currently connected controllers
    ///
    /// Note: To also get controllers that have been disconnected, use
    /// [`all_controllers`]
    fn controllers(&self) -> Vec<Id> {
        self.all_controllers()
            .into_iter()
            .filter(|id| self.controller(id).map(|c| c.info.connected).unwrap_or(false))
            .collect()
    }
    /// Get the [`ID`]s for all controllers
    ///
    /// Note: This also contains controllers that are currently disconnected
    fn all_controllers(&self) -> Vec<Id>;
    /// Retrieve a controller by its [`ID`], or `None` if it does not exist
    fn controller(&self, id: &Id) -> Option<Controller>;
}
