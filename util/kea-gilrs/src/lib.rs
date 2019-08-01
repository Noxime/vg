use kea::input;
use gilrs;
use std::sync::{Mutex, Arc};
use std::time::SystemTime;

#[derive(Default)]
struct KeyState {
    // left joy
    w: bool,
    a: bool,
    s: bool,
    d: bool,
    // right joy
    i: bool,
    j: bool,
    k: bool,
    l: bool,
    // dpad
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    // buttons
    fa: bool, // E
    fb: bool, // O
    fx: bool, // Q
    fy: bool, // U
    // shoulders
    lb: bool, // ctrl/alt/cmd
    lt: bool, // shift
    rb: bool,
    rt: bool,
    // misc
    start: bool, // R
    select: bool, // T
}

pub struct Input {
    gilrs: Mutex<gilrs::Gilrs>,
    map: Mutex<std::collections::HashMap<input::Id, gilrs::GamepadId>>,
    mapping: input::KeyboardMapping,
    kb: (SystemTime, KeyState),
}

impl Input {
    pub fn new() -> Input {
        Input {
            gilrs: Mutex::new(gilrs::Gilrs::new().expect("Couldnt init gilrs")),
            map: Mutex::new(std::collections::HashMap::new()),
            mapping: input::KeyboardMapping::default(),
            kb: (SystemTime::now(), KeyState::default()),
        }
    }

    pub fn update(&self) {
        let ids: Vec<gilrs::GamepadId> = {
            let mut gilrs = self.gilrs.lock().unwrap();
            while let Some(_) = gilrs.next_event() {}
            gilrs.gamepads().map(|(id, _)| id).collect()
        };
        let mut map = self.map.lock().unwrap();
        for id in ids {
            let _ = map.insert(id.into(), id);
        }
    }

    pub fn event(&mut self, key: input::Key, state: bool) {
        if self.mapping.start.contains(&key) { self.kb.1.start = state }
        if self.mapping.select.contains(&key) { self.kb.1.select = state }

        if self.mapping.left_joy_up.contains(&key) { self.kb.1.w = state }
        if self.mapping.left_joy_down.contains(&key) { self.kb.1.s = state }
        if self.mapping.left_joy_left.contains(&key) { self.kb.1.a = state }
        if self.mapping.left_joy_right.contains(&key) { self.kb.1.d = state }

        if self.mapping.right_joy_up.contains(&key) { self.kb.1.i = state }
        if self.mapping.right_joy_down.contains(&key) { self.kb.1.k = state }
        if self.mapping.right_joy_left.contains(&key) { self.kb.1.j = state }
        if self.mapping.right_joy_right.contains(&key) { self.kb.1.l = state }

        if self.mapping.dpad_up.contains(&key) { self.kb.1.up = state }
        if self.mapping.dpad_down.contains(&key) { self.kb.1.down = state }
        if self.mapping.dpad_left.contains(&key) { self.kb.1.left = state }
        if self.mapping.dpad_right.contains(&key) { self.kb.1.right = state }

        if self.mapping.buttons_up.contains(&key) { self.kb.1.fy = state }
        if self.mapping.buttons_down.contains(&key) { self.kb.1.fa = state }
        if self.mapping.buttons_left.contains(&key) { self.kb.1.fx = state }
        if self.mapping.buttons_right.contains(&key) { self.kb.1.fb = state }

        if self.mapping.left_shoulder_trigger.contains(&key) { self.kb.1.lt = state }
        if self.mapping.left_shoulder_bumper.contains(&key) { self.kb.1.lb = state }

        if self.mapping.right_shoulder_trigger.contains(&key) { self.kb.1.rt = state }
        if self.mapping.right_shoulder_bumper.contains(&key) { self.kb.1.rb = state }

        self.kb.0 = SystemTime::now();
    }

}

const KB_ID: input::Id = 1552525;

impl kea::Input for Input {
    fn default(&self) -> Option<input::Id> {
        let gilrs = self.gilrs.lock().unwrap();
        let mut latest = (std::time::SystemTime::UNIX_EPOCH, None);
        let map = self.map.lock().unwrap();
        for (id, g) in map.iter() {
            for (_, data) in gilrs.gamepad(*g).state().buttons() {
                if data.timestamp() >= latest.0 {
                    latest = (data.timestamp(), Some(id));
                }
            }
            for (_, data) in gilrs.gamepad(*g).state().axes() {
                if data.timestamp() >= latest.0 {
                    latest = (data.timestamp(), Some(id));
                }
            }
        }

        // keyboard
        if latest.0 < self.kb.0 {
            return Some(KB_ID)
        }

        latest.1.map(|x| *x)
    }

    fn all_controllers(&self) -> Vec<input::Id> {
        let mut x: Vec<input::Id> = self.map.lock().unwrap().keys().map(|v| *v).collect();
        x.push(KB_ID); // keyboard
        x
    }

    fn controller(&self, id: &input::Id) -> Option<input::Controller> {
        use input::*;

        // keyboard
        if *id == KB_ID {
            let state = &self.kb.1;
            return Some(input::Controller {
                info: input::Info {
                    id: KB_ID,
                    name: "Keyboard".into(),
                    power: input::Power::Wired,
                    connected: true,
                },
                start: state.start.into(),
                select: state.select.into(),
                left_joy: input::Joy {
                    x: match (state.a, state.d) {
                        (true, false) => -1.0,
                        (false, true) =>  1.0,
                        _ => 0.0
                    },
                    y: match (state.s, state.w) {
                        (true, false) => -1.0,
                        (false, true) =>  1.0,
                        _ => 0.0
                    },
                },
                right_joy: input::Joy {
                    x: match (state.j, state.l) {
                        (true, false) => -1.0,
                        (false, true) =>  1.0,
                        _ => 0.0
                    },
                    y: match (state.k, state.i) {
                        (true, false) => -1.0,
                        (false, true) =>  1.0,
                        _ => 0.0
                    },
                },
                dpad: input::Buttons {
                    up: state.up.into(),
                    down: state.down.into(),
                    left: state.left.into(),
                    right: state.right.into(),
                },
                buttons: input::Buttons {
                    up: state.fy.into(),
                    down: state.fa.into(),
                    left: state.fx.into(),
                    right: state.fb.into(),
                },
                left_shoulder: input::Shoulder {
                    trigger: state.lt.into(),
                    bumper: state.lb.into(),
                },
                right_shoulder: input::Shoulder {
                    trigger: state.rt.into(),
                    bumper: state.rb.into(),
                },
            })
        }

        self.map.lock().unwrap().get(id).map(|id| {
            let gilrs = self.gilrs.lock().unwrap();
            let pad = gilrs.gamepad(*id);
            Controller {
                info: Info {
                    name: pad.name().to_string(),
                    power: match pad.power_info() {
                        gilrs::PowerInfo::Unknown => Power::Unknown,
                        gilrs::PowerInfo::Wired => Power::Wired,
                        gilrs::PowerInfo::Discharging(v) => Power::Discharging(v as f32 / 100.0),
                        gilrs::PowerInfo::Charging(v) => Power::Charging(v as f32 / 100.0),
                        gilrs::PowerInfo::Charged => Power::Charged,
                    },
                    connected: true,
                    id: pad.id().into(),
                },
                start: pad.is_pressed(gilrs::Button::Start).into(),
                select: pad.is_pressed(gilrs::Button::Select).into(),
                left_joy: Joy {
                    x: pad.value(gilrs::Axis::LeftStickX),
                    y: pad.value(gilrs::Axis::LeftStickY),
                },
                right_joy: Joy {
                    x: pad.value(gilrs::Axis::RightStickX),
                    y: pad.value(gilrs::Axis::RightStickY),
                },
                dpad: Buttons {
                    up: pad.is_pressed(gilrs::Button::DPadUp).into(),
                    down: pad.is_pressed(gilrs::Button::DPadDown).into(),
                    left: pad.is_pressed(gilrs::Button::DPadLeft).into(),
                    right: pad.is_pressed(gilrs::Button::DPadRight).into(),
                },
                buttons: Buttons {
                    up: pad.is_pressed(gilrs::Button::North).into(),
                    down: pad.is_pressed(gilrs::Button::South).into(),
                    left: pad.is_pressed(gilrs::Button::West).into(),
                    right: pad.is_pressed(gilrs::Button::East).into(),
                },
                left_shoulder: Shoulder {
                    bumper: pad.is_pressed(gilrs::Button::LeftTrigger).into(),
                    trigger: pad.value(gilrs::Axis::LeftZ).into(),
                },
                right_shoulder: Shoulder {
                    bumper: pad.is_pressed(gilrs::Button::RightTrigger).into(),
                    trigger: pad.value(gilrs::Axis::RightZ).into(),
                },
            }
        })
    }

    fn pointers(&self) -> Vec<input::Pointer> {
        unimplemented!()
    }

    fn mapping(&self) -> input::KeyboardMapping {
        self.mapping.clone()
    }

    fn set_mapping(&mut self, mapping: input::KeyboardMapping) {
        self.mapping = mapping;
    }
}

