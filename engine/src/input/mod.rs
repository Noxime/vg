//! Input (gamepad, keyboard, mouse, touch)

use std::collections::HashMap;

// may require tweaking, also look into making this ocnfigurable
// controls deadzoning AND analog-to-digital conversion
const DEADZONE: f32 = 0.05;

use crate::{Coord, Event, Time};

pub mod gamepad;
pub mod keyboard;
pub mod mouse;
pub mod touch;

use keyboard::{Key, Mods};
use mouse::Button as MouseButton;

use gamepad::{Id as GamepadId, Info as GamepadInfo, State as GamepadState};

pub struct Input {
    mapping: VirtualMapping,
    kb_cache: [Digital; 160],
    kb_time: Time,
    m_pos: Coord,
    m_cache: [Digital; 3],
    m_scroll: f32,
    gp_list: HashMap<GamepadId, (GamepadInfo, GamepadState)>,
    gp_time: Time,
    gp_latest: GamepadId,
}

impl Input {
    #[doc(hidden)]
    pub fn new() -> Input {
        Input {
            m_cache: [Digital::Up; 3],
            kb_cache: [Digital::Up; 160],
            kb_time: Time::epoch(),
            gp_time: Time::epoch(),
            gp_latest: GamepadId(0),
            mapping: Default::default(),
            m_pos: Default::default(),
            m_scroll: Default::default(),
            gp_list: Default::default(),
        }
    }

    #[doc(hidden)]
    pub fn handle(&mut self, now: Time, e: &Event) {
        match e {
            Event::Keyboard(e) => {
                self.kb_time = now;
                match e {
                    keyboard::Event::Up(k) => {
                        self.kb_cache[*k as usize] = Digital::Released
                    }
                    keyboard::Event::Down(k) => {
                        self.kb_cache[*k as usize] = Digital::Pressed
                    }
                    _ => (),
                }
            }
            Event::Mouse(e) => match e {
                mouse::Event::Moved(c) => self.m_pos = *c,
                mouse::Event::Up(b) => {
                    self.m_cache[*b as usize] = Digital::Released
                }
                mouse::Event::Down(b) => {
                    self.m_cache[*b as usize] = Digital::Pressed
                }
                mouse::Event::Scroll(s) => self.m_scroll += s,
                _ => (),
            },
            Event::Gamepad { id, ev } => {
                self.gp_time = now;
                self.gp_latest = *id;

                if let Some(v) = self.gp_list.get_mut(id) {
                    match ev {
                        gamepad::Event::Connected {
                            name,
                            force_feedback,
                        } => {
                            v.0.name = name.clone();
                            v.0.force_feedback = *force_feedback;
                            v.0.connected = true;
                        }
                        gamepad::Event::Disconnected => v.0.connected = false,
                        gamepad::Event::Power(p) => v.0.power = *p,
                        gamepad::Event::Axis(axis, value) => {
                            v.1.axes[*axis as usize] = *value
                        }
                        gamepad::Event::Button(button, value) => {
                            v.1.buttons[*button as usize] = *value
                        }
                    }
                } else if let gamepad::Event::Connected {
                    name,
                    force_feedback,
                } = ev
                {
                    self.gp_list.insert(
                        *id,
                        (
                            GamepadInfo {
                                name: name.clone(),
                                force_feedback: *force_feedback,
                                connected: true,
                                power: gamepad::Power::Unknown,
                            },
                            GamepadState::default(),
                        ),
                    );
                }
            }
            _ => (),
        }
    }

    /// Update the internal cache to new values every frame, like pressed turns
    /// to down and released turns to up
    #[doc(hidden)]
    pub fn frame(&mut self) {
        self.m_scroll = 0.0;

        let map = |b: &mut Digital| {
            *b = match b {
                Digital::Pressed => Digital::Down,
                Digital::Released => Digital::Up,
                _ => return,
            };
        };

        for key in self.kb_cache.iter_mut() {
            map(key)
        }

        for button in self.m_cache.iter_mut() {
            map(button)
        }

        for state in self.gp_list.values_mut() {
            for button in state.1.buttons.iter_mut() {
                map(button)
            }
        }
    }
    /// A virtual gamepad that may be backed by either a keyboard or an actual
    ///  gamepad.
    ///
    /// This is useful if you want to make your input handling simple
    /// and not deal with multiple input types. The device chosen to back
    /// this is the one that received the latest input
    ///
    /// See [`Input::mapping`] to configure keyboard-to-gamepad mapping
    /// for your usecase
    pub fn input(&self) -> GamepadState {
        if self.gp_time > self.kb_time {
            return self.gamepad(self.gp_latest).unwrap_or_default();
        }

        use gamepad::{Axis, Button};

        let m = self.mapping();
        let mut state = GamepadState::default();

        let axes = [
            (Axis::LeftX, &m.left_joy_right, &m.left_joy_left),
            (Axis::LeftY, &m.left_joy_up, &m.left_joy_down),
            (Axis::RightX, &m.right_joy_right, &m.right_joy_left),
            (Axis::RightY, &m.right_joy_up, &m.right_joy_down),
            (Axis::LeftZ, &m.left_shoulder_trigger, &vec![]),
            (Axis::RightZ, &m.right_shoulder_trigger, &vec![]),
            (Axis::DPadX, &m.dpad_right, &m.dpad_left),
            (Axis::DPadY, &m.dpad_up, &m.dpad_down),
        ];

        let buttons = [
            (Button::Select, &m.select),
            (Button::Start, &m.start),
            (Button::Mode, &m.mode),
            (Button::DPadUp, &m.dpad_up),
            (Button::DPadRight, &m.dpad_right),
            (Button::DPadDown, &m.dpad_down),
            (Button::DPadLeft, &m.dpad_left),
            (Button::FaceUp, &m.face_up),
            (Button::FaceRight, &m.face_right),
            (Button::FaceDown, &m.face_down),
            (Button::FaceLeft, &m.face_left),
            (Button::LeftBumper, &m.left_shoulder_bumper),
            (Button::RightBumper, &m.right_shoulder_bumper),
            (Button::LeftTrigger, &m.left_shoulder_trigger),
            (Button::RightTrigger, &m.right_shoulder_trigger),
            (Button::LeftThumb, &m.left_thumb),
            (Button::RightThumb, &m.right_thumb),
        ];

        for (axis, pos, neg) in axes.iter() {
            if combined(pos.iter().map(|k| self.key(*k))).down() {
                state.axes[*axis as usize].0 += 1.0;
            }
            if combined(neg.iter().map(|k| self.key(*k))).down() {
                state.axes[*axis as usize].0 -= 1.0;
            }
        }

        for (button, keys) in buttons.iter() {
            state.buttons[*button as usize] =
                combined(keys.iter().map(|k| self.key(*k)));
        }

        state
    }

    /// Set the mapping used for [`Input::input`]
    pub fn set_mapping(&mut self, map: VirtualMapping) {
        self.mapping = map;
    }
    /// Get the currently active mapping for [`Input::input`]
    pub fn mapping(&self) -> &VirtualMapping {
        &self.mapping
    }

    /// Get the state of a key
    pub fn key(&self, key: Key) -> Digital {
        self.kb_cache[key as usize]
    }

    /// Get the state of the modifier keys (shift, ctrl etc)
    pub fn mods(&self) -> Mods {
        Mods {
            left_shift: self.kb_cache[Key::LShift as usize],
            left_ctrl: self.kb_cache[Key::LCtrl as usize],
            left_alt: self.kb_cache[Key::LAlt as usize],
            left_mode: self.kb_cache[Key::LMode as usize],

            right_shift: self.kb_cache[Key::RShift as usize],
            right_ctrl: self.kb_cache[Key::RCtrl as usize],
            right_alt: self.kb_cache[Key::RAlt as usize],
            right_mode: self.kb_cache[Key::RMode as usize],
        }
    }

    /// Get the position of the mouse, with 0,0 being center and 1, 1 being
    /// top-right corner
    pub fn mouse_pos(&self) -> Coord {
        self.m_pos
    }

    /// Get the state of a mouse button
    pub fn mouse_button(&self, button: MouseButton) -> Digital {
        self.m_cache[button as usize]
    }

    /// Get the total mouse scrolling for this frame
    pub fn mouse_scroll(&self) -> f32 {
        self.m_scroll
    }

    /// Get a list of all known gamepads
    pub fn gamepads(&self) -> impl Iterator<Item = GamepadId> {
        (0..self.gp_list.len()).map(|i| GamepadId(i))
    }

    /// Get the information about a gamepad, if it exists
    ///
    /// Note: This includes disconnected controllers, so make sure to check the
    /// [`GamepadInfo.connected`] field
    pub fn info(&self, id: GamepadId) -> Option<GamepadInfo> {
        self.gp_list.get(&id).map(|x| x.0.clone())
    }

    /// Get the state of a gamepad, if it exists
    ///
    /// Note: This will return the state even if the controller has been
    /// disconnected. You can check which controllers are currently connected
    /// with the [`Input::info`] function
    pub fn gamepad(&self, id: GamepadId) -> Option<GamepadState> {
        self.gp_list.get(&id).map(|x| x.1.clone())
    }
}

/// combines multiple digital inputs to act as one key
///
/// returns Up if empty iterator
fn combined(mut i: impl Iterator<Item = Digital>) -> Digital {
    let mut prev = match i.next() {
        Some(x) => x,
        None => return Digital::Up,
    };

    for next in i.next() {
        use Digital::*;
        prev = match (prev, next) {
            (Up, x) | (x, Up) => x,
            (Down, _) | (_, Down) => Down,
            (Pressed, Released) | (Released, Pressed) => Down,
            _ => prev,
        }
    }
    prev
}

/// The mapping between a keyboard and a gamepad, used for [`Input::input`]
///
/// [`Default`] mapping is as follows
/// 
/// | Gamepad     | Keyboard |
/// |-------------|----------|
/// | Start       | Escape   |
/// | Select      | Enter    |
/// | Left stick  | WASD     |
/// | Right stick | Arrows   |
/// | Face-left   | R        |
/// | Face-right  | Space    |
/// | Face-up     | Q        |
/// | Face-down   | E        |
/// | Triggers    | Shifts   |
/// | Bumpers     | Ctrls    |
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VirtualMapping {
    pub start: Vec<Key>,
    pub select: Vec<Key>,
    pub mode: Vec<Key>,

    pub left_joy_up: Vec<Key>,
    pub left_joy_down: Vec<Key>,
    pub left_joy_left: Vec<Key>,
    pub left_joy_right: Vec<Key>,

    pub right_joy_up: Vec<Key>,
    pub right_joy_down: Vec<Key>,
    pub right_joy_left: Vec<Key>,
    pub right_joy_right: Vec<Key>,

    pub left_thumb: Vec<Key>,
    pub right_thumb: Vec<Key>,

    pub dpad_up: Vec<Key>,
    pub dpad_down: Vec<Key>,
    pub dpad_left: Vec<Key>,
    pub dpad_right: Vec<Key>,

    pub face_up: Vec<Key>,
    pub face_down: Vec<Key>,
    pub face_left: Vec<Key>,
    pub face_right: Vec<Key>,

    pub left_shoulder_trigger: Vec<Key>,
    pub left_shoulder_bumper: Vec<Key>,
    pub right_shoulder_trigger: Vec<Key>,
    pub right_shoulder_bumper: Vec<Key>,
}

impl VirtualMapping {
    fn empty() -> VirtualMapping {
        VirtualMapping {
            start: vec![],
            select: vec![],
            mode: vec![],

            left_joy_up: vec![],
            left_joy_down: vec![],
            left_joy_left: vec![],
            left_joy_right: vec![],

            right_joy_up: vec![],
            right_joy_down: vec![],
            right_joy_left: vec![],
            right_joy_right: vec![],

            left_thumb: vec![],
            right_thumb: vec![],

            dpad_up: vec![],
            dpad_down: vec![],
            dpad_left: vec![],
            dpad_right: vec![],

            face_up: vec![],
            face_down: vec![],
            face_left: vec![],
            face_right: vec![],

            left_shoulder_trigger: vec![],
            left_shoulder_bumper: vec![],
            right_shoulder_trigger: vec![],
            right_shoulder_bumper: vec![],
        }
    }
}

impl Default for VirtualMapping {
    fn default() -> VirtualMapping {
        VirtualMapping {
            start: vec![Key::Escape],
            select: vec![Key::Enter],

            left_joy_up: vec![Key::W],
            left_joy_down: vec![Key::S],
            left_joy_left: vec![Key::A],
            left_joy_right: vec![Key::D],

            right_joy_up: vec![Key::Up],
            right_joy_down: vec![Key::Down],
            right_joy_left: vec![Key::Left],
            right_joy_right: vec![Key::Right],

            face_up: vec![Key::Q],
            face_down: vec![Key::E],
            face_left: vec![Key::R],
            face_right: vec![Key::Space],

            left_shoulder_trigger: vec![Key::LShift],
            left_shoulder_bumper: vec![Key::LCtrl],
            right_shoulder_trigger: vec![Key::RShift],
            right_shoulder_bumper: vec![Key::RCtrl],

            ..VirtualMapping::empty()
        }
    }
}

/// A digital state of a button or a key
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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
}

/// An analog input, like the joysticks of a gamepad
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
