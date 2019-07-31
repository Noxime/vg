use kea::Api;
use kea_glium::glutin;

// use placeholder_audio as kea_cpal;

struct Wasm {
    renderer: kea_glium::Renderer,
    input: kea_gilrs::Input,
    audio: kea_cpal::Audio,
    events: glutin::EventsLoop,
    closing: bool,
}

impl Api for Wasm {
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

use wasm_bindgen::prelude::*;

// // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// // allocator.
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

// #[wasm_bindgen]
pub fn main() {
    console_error_panic_hook::set_once();
    println!("Running Kea engine on WASM");

    let (renderer, events) = kea_glium::Renderer::new();

    game::run(Wasm {
        renderer,
        events,
        closing: false,
        input: kea_gilrs::Input::new(),
        audio: kea_cpal::Audio::new(),
    })
}