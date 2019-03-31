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

struct Input(
    Mutex<gilrs::Gilrs>,
    Mutex<std::collections::HashMap<input::Id, gilrs::GamepadId>>,
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
    fn default(&self) -> input::Id {
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
        *latest.1.unwrap()
    }

    fn all_controllers(&self) -> Vec<input::Id> {
        self.update();
        self.1.lock().unwrap().keys().map(|v| *v).collect()
    }

    fn controller(&self, id: &input::Id) -> Option<input::Controller> {
        self.update();
        use input::*;

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
    // fn controllers(&self) -> Vec<input::Controller> {

    //     let mut gilrs = self.0.lock().unwrap();

    //     while let Some(_) = gilrs.next_event() {}
    //     // TODO: GilRs does not expose buttons as analog (probably because
    //     // analog buttons are rare, but not non-existent, see DualShock 2).
    //     // Anyway, maybe some day we can fix this? out of scope for now.
    //     // noxim - 2019-03-31
    //     gilrs.gamepads().map(|(_id, pad)| {
    //         use input::*;
    //         let state = pad.state();

    //     }).collect()
    // }
}

fn main() {
    let (renderer, mut events) = kea_dev::Renderer::new();
    let platform_state = Arc::new(Mutex::new(ApiState {
        should_close: false,
    }));
    let platform = Api(platform_state.clone());
    let input = Input(
        Mutex::new(gilrs::Gilrs::new().unwrap()),
        Mutex::new(std::collections::HashMap::new()),
    );

    let poll = Box::new(move || {
        events.poll_events(|e| {
            use glutin::{Event, WindowEvent};
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
                _ => (),
            }
        })
    });

    kea::run(platform, renderer, input, poll, &game::game);
}
