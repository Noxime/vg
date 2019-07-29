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

        self.events.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent { event: glutin::WindowEvent::CloseRequested, .. } => closing = true,
                _ => (),
            }
        });

        self.closing = closing;

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