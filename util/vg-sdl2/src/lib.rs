// a vg graphics, input and audio backend backed by SDL2

mod texture;
mod surface;
pub use texture::Texture;
pub use surface::Surface;

pub struct Renderer {
    context: sdl2::Sdl,
    video: sdl2::VideoSubsystem,
    surface: Surface,
    events: sdl2::EventPump,
}

impl Renderer {
    pub fn new() -> Renderer {
        let context = sdl2::init().unwrap();
        println!("context: {}", sdl2::get_error());
        let video = context.video().unwrap();
        println!("video: {}", sdl2::get_error());

        let display = video
            .window("vg", 800, 600)
            .resizable()
            .build()
            .unwrap();

        println!("display: {}", sdl2::get_error());

        let mut canvas = display.into_canvas().present_vsync().build().unwrap();
        println!("SDL: {} (flags: {:#b})", canvas.info().name, canvas.info().flags);
        println!("canvas: {}", sdl2::get_error());

        let start = std::time::Instant::now();
        let events = context.event_pump().unwrap();
        match sdl2::get_error().as_str() {
            e if !e.is_empty() => println!("Error: {}", e),
            _ => ()
        }

        canvas.set_draw_color(sdl2::pixels::Color::RGBA(255, 127, 0, 255));
        canvas.clear();

        Renderer {
            context,
            video,
            events,
            surface: Surface { canvas }
        }
    }

    pub fn poll(&mut self) {
        println!("Poll");
        for event in self.events.poll_iter() {
            use sdl2::event::Event;
            println!("event: {:#?}", event);
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

impl Drop for Renderer {
    fn drop(&mut self) {
        let _ = self.context.video();
        println!("{}", self.video.current_video_driver());
    }
}

impl vg::Renderer for Renderer {
    const NAME: &'static str = "SDL2";
    type Texture = Texture;
    type Surface = Surface;

    fn surface(&mut self) -> &mut Surface {
        &mut self.surface
    }
}