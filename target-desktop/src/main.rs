use kea::Api;
use kea_glium::glutin;

struct Desktop {
    renderer: kea_glium::Renderer,
    input: kea_gilrs::Input,
    audio: kea_cpal::Audio,
    events: glutin::EventsLoop,
    closing: bool,
}

impl Api for Desktop {
    type R = kea_glium::Renderer;
    type I = kea_gilrs::Input;
    type A = kea_cpal::Audio;

    fn poll(&mut self) {
        let mut closing = false;
        let mut keys = vec![];

        self.events.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent { event: glutin::WindowEvent::CloseRequested, .. } => closing = true,
                glutin::Event::DeviceEvent { event: glutin::DeviceEvent::Key(glutin::KeyboardInput { virtual_keycode: Some(k), state, .. }), .. } => {
                    use glutin::VirtualKeyCode as Kc;
                    use kea::input::Key;
                    if let Some(k) = match k {
                        Kc::Key0 => Some(Key::Key0),
                        Kc::Key1 => Some(Key::Key1),
                        Kc::Key2 => Some(Key::Key2),
                        Kc::Key3 => Some(Key::Key3),
                        Kc::Key4 => Some(Key::Key4),
                        Kc::Key5 => Some(Key::Key5),
                        Kc::Key6 => Some(Key::Key6),
                        Kc::Key7 => Some(Key::Key7),
                        Kc::Key8 => Some(Key::Key8),
                        Kc::Key9 => Some(Key::Key9),
                        Kc::A => Some(Key::A),
                        Kc::B => Some(Key::B),
                        Kc::C => Some(Key::C),
                        Kc::D => Some(Key::D),
                        Kc::E => Some(Key::E),
                        Kc::F => Some(Key::F),
                        Kc::G => Some(Key::G),
                        Kc::H => Some(Key::H),
                        Kc::I => Some(Key::I),
                        Kc::J => Some(Key::J),
                        Kc::K => Some(Key::K),
                        Kc::L => Some(Key::L),
                        Kc::M => Some(Key::M),
                        Kc::N => Some(Key::N),
                        Kc::O => Some(Key::O),
                        Kc::P => Some(Key::P),
                        Kc::Q => Some(Key::Q),
                        Kc::R => Some(Key::R),
                        Kc::S => Some(Key::S),
                        Kc::T => Some(Key::T),
                        Kc::U => Some(Key::U),
                        Kc::V => Some(Key::V),
                        Kc::W => Some(Key::W),
                        Kc::X => Some(Key::X),
                        Kc::Y => Some(Key::Y),
                        Kc::Z => Some(Key::Z),
                        Kc::Escape => Some(Key::Esc),
                        Kc::F1 => Some(Key::F1),
                        Kc::F2 => Some(Key::F2),
                        Kc::F3 => Some(Key::F3),
                        Kc::F4 => Some(Key::F4),
                        Kc::F5 => Some(Key::F5),
                        Kc::F6 => Some(Key::F6),
                        Kc::F7 => Some(Key::F7),
                        Kc::F8 => Some(Key::F8),
                        Kc::F9 => Some(Key::F9),
                        Kc::F10 => Some(Key::F10),
                        Kc::F11 => Some(Key::F11),
                        Kc::F12 => Some(Key::F12),
                        Kc::Up => Some(Key::Up),
                        Kc::Down => Some(Key::Down),
                        Kc::Left => Some(Key::Left),
                        Kc::Right => Some(Key::Right),
                        Kc::Back => Some(Key::Back),
                        Kc::Return => Some(Key::Enter),
                        Kc::Space => Some(Key::Space),
                        Kc::Period => Some(Key::Period),
                        Kc::Comma => Some(Key::Comma),
                        Kc::Minus => Some(Key::Minus),
                        Kc::LShift => Some(Key::LShift),
                        Kc::RShift => Some(Key::RShift),
                        Kc::LAlt => Some(Key::LAlt),
                        Kc::RAlt => Some(Key::RAlt),
                        Kc::LControl => Some(Key::LCtrl),
                        Kc::RControl => Some(Key::RCtrl),
                        Kc::LWin => Some(Key::LSuper),
                        Kc::RWin => Some(Key::RSuper),
                        _ => None
                    } {
                        keys.push((k, state == glutin::ElementState::Pressed))
                    }
                },
                _ => (),
            }
        });

        self.closing = closing;

        for (c, s) in keys {
            self.input.event(c, s)
        }
        self.input.update()
    }

    fn exit(&self) -> bool {
        self.closing
    }

    fn renderer<'a>(&'a mut self) -> &'a mut kea_glium::Renderer {
        &mut self.renderer
    }

    fn input<'a>(&'a mut self) -> &'a mut kea_gilrs::Input {
        &mut self.input
    }

    fn audio<'a>(&'a mut self) -> &'a mut kea_cpal::Audio {
        &mut self.audio
    }
}

fn main() {
    let (renderer, events) = kea_glium::Renderer::new();
    game::run(Desktop {
        renderer,
        events,
        closing: false,
        input: kea_gilrs::Input::new(),
        audio: kea_cpal::Audio::new(),
    })
}