extern crate audio_ears;
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


fn main() {
    let (renderer, mut events) = kea_dev::Renderer::new();
    let platform_state = Arc::new(Mutex::new(ApiState {
        should_close: false,
    }));
    let platform = Api(platform_state.clone());
    let keyboard = Arc::new(Mutex::new((
        std::time::SystemTime::UNIX_EPOCH,
        KeyState::default(),
    )));

    let input = Input(
        Mutex::new(gilrs::Gilrs::new().unwrap()),
        Mutex::new(std::collections::HashMap::new()),
        keyboard.clone(),
    );

    let poll = Box::new(move || {
        events.poll_events(|e| {
            use glutin::{
                ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent,
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
                    event:
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
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
                        VirtualKeyCode::LControl | VirtualKeyCode::LWin | VirtualKeyCode::LAlt => {
                            keyboard.lock().unwrap().1.lb = s
                        }
                        VirtualKeyCode::RControl | VirtualKeyCode::RWin | VirtualKeyCode::RAlt => {
                            keyboard.lock().unwrap().1.rb = s
                        }

                        VirtualKeyCode::R => keyboard.lock().unwrap().1.start = s,
                        VirtualKeyCode::T => keyboard.lock().unwrap().1.select = s,

                        _ => (),
                    }
                    keyboard.lock().unwrap().0 = std::time::SystemTime::now();
                }
                _ => (),
            }
        })
    });

    let audio = audio_ears::Audio::new();

    kea::run(platform, renderer, input, audio, poll, &game::game);
}
