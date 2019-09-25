// a kea graphics, input and audio backend backed by SDL2

mod texture;
mod surface;
pub use texture::Texture;
pub use surface::Surface;

pub struct Renderer {
    surface: Surface,
    events: sdl2::EventPump,
}

impl Renderer {
    pub fn new() -> Renderer {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let display = video_subsystem
            .window("Kea", 800, 600)
            .resizable()
            .build()
            .unwrap();

        let mut canvas = display.into_canvas().build().unwrap();
        println!("SDL: {} (flags: {:#b})", canvas.info().name, canvas.info().flags);

        let start = std::time::Instant::now();
        let events = sdl_context.event_pump().unwrap();
        match sdl2::get_error().as_str() {
            e if !e.is_empty() => println!("Error: {}", e),
            _ => ()
        }

        Renderer {
            events,
            surface: Surface { canvas }
        }
    }

    pub fn poll(&mut self) {
        for event in self.events.poll_iter() {
            use sdl2::event::Event;
            match event {
                Event::Quit { .. } => {
                    println!("Engine exit");
                    std::process::exit(0);
                }
                _ => (),
            }
        }
    }
}

impl kea::Renderer for Renderer {
    const NAME: &'static str = "SDL2";
    type Texture = Texture;
    type Surface = Surface;

    fn surface(&mut self) -> &mut Surface {
        &mut self.surface
    }
}