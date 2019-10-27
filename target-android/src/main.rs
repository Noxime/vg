extern crate android_glue;
extern crate game;
extern crate vg;
extern crate vg_glium;
extern crate placeholder_input;

use vg::Api;

struct AndroidHandler {}

impl android_glue::SyncEventHandler for AndroidHandler {
    fn handle(&mut self, event: &android_glue::Event) {
        println!("{:#?}", event);
        match event {
            android_glue::Event::LostFocus => {
                println!("FOCUS LOST: Save game state");
                // TODO: ask game for its state and save it, and set reload flag
                std::process::exit(0);
            }
            _ => (),
        }
    }
}

struct Android {
    renderer: vg_glium::Renderer,
    input: vg_gilrs::Input,
    audio: vg_cpal::Audio,
    events: glutin::EventsLoop,
    closing: bool,
}

impl Api for Android {
    type R = vg_glium::Renderer;
    type I = vg_gilrs::Input;
    type A = vg_cpal::Audio;

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

    fn renderer<'a>(&'a mut self) -> &'a mut vg_glium::Renderer {
        &mut self.renderer
    }

    fn input<'a>(&'a mut self) -> &'a mut vg_gilrs::Input {
        &mut self.input
    }

    fn audio<'a>(&'a mut self) -> &'a mut vg_cpal::Audio {
        &mut self.audio
    }
}

fn main() {
    let (renderer, events) = vg_glium::Renderer::new();
    game::run(Android {
        renderer,
        events,
        closing: false,
        input: placeholder_input::Input,
        audio: vg_cpal::Audio::new(),
    })
}