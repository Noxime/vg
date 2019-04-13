extern crate game;
extern crate gilrs;
extern crate kea;
extern crate kea_dev;

use kea::input;
use kea_dev::glutin;

use std::sync::{Arc, Mutex};

struct Api(Arc<Mutex<ApiState>>);
struct ApiState {
    should_close: bool,
}

impl kea::PlatformApi for Api {
    fn exit(&self) -> bool {
        if let Ok(state) = self.0.lock() {
            return state.should_close;
        }
        false
    }

    fn print(&self, s: &str) {
        println!("{}", s);
    }
}

// TODO: make this uhh not so dumb
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

struct Input(
    Mutex<gilrs::Gilrs>,
    Mutex<std::collections::HashMap<input::Id, gilrs::GamepadId>>,
    Arc<Mutex<(std::time::SystemTime, KeyState)>>,
);
impl Input {
    fn update(&self) {
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
        self.update();
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
        self.update();
        let mut x: Vec<input::Id> = self.1.lock().unwrap().keys().map(|v| *v).collect();
        x.push(!0);
        x
    }

    fn controller(&self, id: &input::Id) -> Option<input::Controller> {
        self.update();
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

fn main() {
    let (renderer, mut events) = kea_dev::Renderer::new();
    let platform_state = Arc::new(Mutex::new(ApiState {
        should_close: false,
    }));
    let platform = Api(platform_state.clone());
    let keyboard = Arc::new(Mutex::new((std::time::SystemTime::UNIX_EPOCH, KeyState::default())));

    let input = Input(
        Mutex::new(gilrs::Gilrs::new().unwrap()),
        Mutex::new(std::collections::HashMap::new()),
        keyboard.clone(),
    );

    let poll = Box::new(move || {
        events.poll_events(|e| {
            use glutin::{
                Event, 
                WindowEvent, 
                DeviceEvent, 
                KeyboardInput, 
                VirtualKeyCode, 
                ElementState
            };
            match e {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    platform_state
                        .lock()
                        .expect("PlatformState lock fail")
                        .should_close = true;
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            state,
                            virtual_keycode: Some(key),
                            ..
                        },
                        ..
                    },
                    ..
                } => {
                    println!("key event: {:?} {:?}", key, state);
                    let s = match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };
                    match key {
                        VirtualKeyCode::W => keyboard.lock().unwrap().1.w = s,
                        VirtualKeyCode::A => keyboard.lock().unwrap().1.a = s,
                        VirtualKeyCode::S => keyboard.lock().unwrap().1.s = s,
                        VirtualKeyCode::D => keyboard.lock().unwrap().1.d = s,

                        VirtualKeyCode::I => keyboard.lock().unwrap().1.i = s,
                        VirtualKeyCode::J => keyboard.lock().unwrap().1.j = s,
                        VirtualKeyCode::K => keyboard.lock().unwrap().1.k = s,
                        VirtualKeyCode::L => keyboard.lock().unwrap().1.l = s,

                        VirtualKeyCode::Up => keyboard.lock().unwrap().1.up = s,
                        VirtualKeyCode::Down => keyboard.lock().unwrap().1.down = s,
                        VirtualKeyCode::Left => keyboard.lock().unwrap().1.left = s,
                        VirtualKeyCode::Right => keyboard.lock().unwrap().1.right = s,

                        VirtualKeyCode::E => keyboard.lock().unwrap().1.fa = s,
                        VirtualKeyCode::Q => keyboard.lock().unwrap().1.fx = s,
                        VirtualKeyCode::O => keyboard.lock().unwrap().1.fb = s,
                        VirtualKeyCode::U => keyboard.lock().unwrap().1.fy = s,

                        VirtualKeyCode::LShift => keyboard.lock().unwrap().1.lt = s,
                        VirtualKeyCode::RShift => keyboard.lock().unwrap().1.rt = s,
                        VirtualKeyCode::LControl |
                        VirtualKeyCode::LWin |
                        VirtualKeyCode::LAlt => keyboard.lock().unwrap().1.lb = s,
                        VirtualKeyCode::RControl |
                        VirtualKeyCode::RWin |
                        VirtualKeyCode::RAlt => keyboard.lock().unwrap().1.rb = s,

                        VirtualKeyCode::R => keyboard.lock().unwrap().1.start = s,
                        VirtualKeyCode::T => keyboard.lock().unwrap().1.select = s,


                        _ => ()
                    }
                    keyboard.lock().unwrap().0 = std::time::SystemTime::now();
                }
                _ => ()
            }
        })
    });

    kea::run(platform, renderer, input, poll, &game::game);
}
