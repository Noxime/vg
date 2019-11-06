extern crate game;
extern crate vg;
extern crate vg_glium;

use vg::Api;

struct Ios {
    renderer: vg_glium::Renderer,
    input: placeholder_input::Input,
    audio: placeholder_audio::Audio,
    events: vg_glium::glutin::EventsLoop,
    closing: bool,
}

impl Api for Ios {
    type R = vg_glium::Renderer;
    type I = placeholder_input::Input;
    type A = placeholder_audio::Audio;
    type T = Time;

    fn poll(&mut self) {
        let mut closing = false;

        self.events.poll_events(|event| {
            match event {
                vg_glium::glutin::Event::WindowEvent { event: vg_glium::glutin::WindowEvent::CloseRequested, .. } => closing = true,
                _ => (),
            }
        });

        self.closing = closing;

        // self.input.update()
    }

    fn exit(&self) -> bool {
        self.closing
    }

    fn renderer<'a>(&'a mut self) -> &'a mut vg_glium::Renderer {
        &mut self.renderer
    }

    fn input<'a>(&'a mut self) -> &'a mut placeholder_input::Input {
        &mut self.input
    }

    fn audio<'a>(&'a mut self) -> &'a mut placeholder_audio::Audio {
        &mut self.audio
    }
}

struct Time(std::time::Instant);
impl vg::Time for Time {
    fn now() -> Time {
        Time(std::time::Instant::now())
    }

    fn elapsed(&self) -> f32 {
        self.0.elapsed().as_secs_f32()
    }
}

#[no_mangle]
pub extern "C" fn ios_main() {
    let (renderer, events) = vg_glium::Renderer::new();
    futures::executor::block_on(game::run(Ios {
        renderer,
        events,
        closing: false,
        input: placeholder_input::Input,
        audio: placeholder_audio::Audio,
    }));
}
