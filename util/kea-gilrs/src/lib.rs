use kea::input;
use gilrs;
use std::sync::{Mutex, Arc};


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

pub struct Input(
    Mutex<gilrs::Gilrs>,
    Mutex<std::collections::HashMap<input::Id, gilrs::GamepadId>>,
    Arc<Mutex<(std::time::SystemTime, KeyState)>>,
);

impl Input {
    pub fn new() -> Input {
        Input(
            Mutex::new(gilrs::Gilrs::new().expect("Couldnt init gilrs")),
            Mutex::new(std::collections::HashMap::new()),
            Arc::new(Mutex::new((std::time::SystemTime::now(), KeyState::default())))
        )
    }

    pub fn update(&self) {
        let ids: Vec<gilrs::GamepadId> = {
            let mut gilrs = self.0.lock().unwrap();
            while let Some(_) = gilrs.next_event() {}
            gilrs.gamepads().map(|(id, _)| id).collect()
        };
        let mut map = self.1.lock().unwrap();
        for id in ids {
            let _ = map.insert(id.into(), id);
        }
    }
}

impl kea::Input for Input {
    fn default(&self) -> Option<input::Id> {
        let gilrs = self.0.lock().unwrap();
        let mut latest = (std::time::SystemTime::UNIX_EPOCH, None);
        let map = self.1.lock().unwrap();
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
        if self.2.lock().unwrap().0 >= latest.0 {
            return Some(!0)
        }

        latest.1.map(|x| *x)
    }

    fn all_controllers(&self) -> Vec<input::Id> {
        let mut x: Vec<input::Id> = self.1.lock().unwrap().keys().map(|v| *v).collect();
        x.push(!0);
        x
    }

    fn controller(&self, id: &input::Id) -> Option<input::Controller> {
        use input::*;

        if *id == !0 {
            let state = &self.2.lock().unwrap().1;
            return Some(Controller {
                info: Info {
                    name: "Keyboard".into(),
                    power: Power::Unknown,
                    id: !0,
                    connected: true,
                },
                start: state.start.into(),
                select: state.select.into(),
                left_joy: Joy {
                    x: if state.a { -1.0 } else { 0.0 } + if state.d { 1.0 } else { 0.0 },
                    y: if state.s { -1.0 } else { 0.0 } + if state.w { 1.0 } else { 0.0 },
                },
                right_joy: Joy {
                    x: if state.j { -1.0 } else { 0.0 } + if state.l { 1.0 } else { 0.0 },
                    y: if state.k { -1.0 } else { 0.0 } + if state.i { 1.0 } else { 0.0 },
                },
                dpad: Buttons {
                    up: state.up.into(),
                    down: state.down.into(),
                    left: state.left.into(),
                    right: state.right.into(),
                },
                buttons: Buttons {
                    up: state.fy.into(),
                    down: state.fa.into(),
                    left: state.fx.into(),
                    right: state.fb.into(),
                },
                left_shoulder: Shoulder {
                    bumper: state.lb.into(),
                    trigger: state.lt.into(),
                },
                right_shoulder: Shoulder {
                    bumper: state.rb.into(),
                    trigger: state.rt.into(),
                },
            })
        }

        self.1.lock().unwrap().get(id).map(|id| {
            let gilrs = self.0.lock().unwrap();
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
        unimplemented!()
    }

    fn set_mapping(&mut self, _: input::KeyboardMapping) {
        unimplemented!()
    }
}

