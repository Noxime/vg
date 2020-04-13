use vg::*;
use vg_glium::glutin;

struct Vg {
    events: glutin::EventsLoop,
    events_queue: std::collections::VecDeque<glutin::Event>,
    start: std::time::Instant,
}

impl Vg {
    fn new(events: glutin::EventsLoop) -> Vg {
        Self {
            events,
            start: std::time::Instant::now(),
            events_queue: Default::default(),
        }
    }
}

impl VgTrait for Vg {
    fn title(&mut self, title: &str) {
        unimplemented!()
    }

    fn resize(&mut self, window: WindowMode) {
        unimplemented!()
    }

    /// Get the next event from the event queue, if possible
    // TODO: Explore the possibilities of making this an async fn or a stream
    fn poll_event(&mut self) -> Option<Event> {
        let mut queue = std::collections::VecDeque::new();
        self.events.poll_events(|event| queue.push_back(event));
        self.events_queue.append(&mut queue);

        if let Some(event) = self.events_queue.pop_front() {
            println!("glutin: {:?}", event);
            return Some(match event {
                glutin::Event::Suspended(false) => vg::Event::FocusGained,
                glutin::Event::Suspended(true) => vg::Event::FocusLost,
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => vg::Event::Exit,
                    _ => return None,
                },
                _ => return None,
            })
        }
        return None
    }

    /// Current time, as high resolution as possible and since the call of game
    /// entrypoint
    fn time(&self) -> Time {
        Time::from_secs(self.start.elapsed().as_secs_f32())
    }

    // Kc::Key0 => Some(Key::Key0),
    // Kc::Key1 => Some(Key::Key1),
    // Kc::Key2 => Some(Key::Key2),
    // Kc::Key3 => Some(Key::Key3),
    // Kc::Key4 => Some(Key::Key4),
    // Kc::Key5 => Some(Key::Key5),
    // Kc::Key6 => Some(Key::Key6),
    // Kc::Key7 => Some(Key::Key7),
    // Kc::Key8 => Some(Key::Key8),
    // Kc::Key9 => Some(Key::Key9),
    // Kc::A => Some(Key::A),
    // Kc::B => Some(Key::B),
    // Kc::C => Some(Key::C),
    // Kc::D => Some(Key::D),
    // Kc::E => Some(Key::E),
    // Kc::F => Some(Key::F),
    // Kc::G => Some(Key::G),
    // Kc::H => Some(Key::H),
    // Kc::I => Some(Key::I),
    // Kc::J => Some(Key::J),
    // Kc::K => Some(Key::K),
    // Kc::L => Some(Key::L),
    // Kc::M => Some(Key::M),
    // Kc::N => Some(Key::N),
    // Kc::O => Some(Key::O),
    // Kc::P => Some(Key::P),
    // Kc::Q => Some(Key::Q),
    // Kc::R => Some(Key::R),
    // Kc::S => Some(Key::S),
    // Kc::T => Some(Key::T),
    // Kc::U => Some(Key::U),
    // Kc::V => Some(Key::V),
    // Kc::W => Some(Key::W),
    // Kc::X => Some(Key::X),
    // Kc::Y => Some(Key::Y),
    // Kc::Z => Some(Key::Z),
    // Kc::Escape => Some(Key::Esc),
    // Kc::F1 => Some(Key::F1),
    // Kc::F2 => Some(Key::F2),
    // Kc::F3 => Some(Key::F3),
    // Kc::F4 => Some(Key::F4),
    // Kc::F5 => Some(Key::F5),
    // Kc::F6 => Some(Key::F6),
    // Kc::F7 => Some(Key::F7),
    // Kc::F8 => Some(Key::F8),
    // Kc::F9 => Some(Key::F9),
    // Kc::F10 => Some(Key::F10),
    // Kc::F11 => Some(Key::F11),
    // Kc::F12 => Some(Key::F12),
    // Kc::Up => Some(Key::Up),
    // Kc::Down => Some(Key::Down),
    // Kc::Left => Some(Key::Left),
    // Kc::Right => Some(Key::Right),
    // Kc::Back => Some(Key::Back),
    // Kc::Return => Some(Key::Enter),
    // Kc::Space => Some(Key::Space),
    // Kc::Period => Some(Key::Period),
    // Kc::Comma => Some(Key::Comma),
    // Kc::Minus => Some(Key::Minus),
    // Kc::LShift => Some(Key::LShift),
    // Kc::RShift => Some(Key::RShift),
    // Kc::LAlt => Some(Key::LAlt),
    // Kc::RAlt => Some(Key::RAlt),
    // Kc::LControl => Some(Key::LCtrl),
    // Kc::RControl => Some(Key::RCtrl),
    // Kc::LWin => Some(Key::LSuper),
    // Kc::RWin => Some(Key::RSuper),
}

fn main() {
    let (gfx, events) = vg_glium::Gfx::new();
    let vg = vg::Vg::new(Box::new(Vg::new(events)));
    let gfx = vg::Gfx::new(Box::new(gfx));
    let sfx = vg::Sfx::new(Box::new(vg_cpal::Sfx {}));

    futures::executor::block_on(game::run(vg, gfx, sfx));
}
