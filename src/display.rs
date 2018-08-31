extern crate gl;
extern crate glutin;

use self::glutin::dpi::*;
use self::glutin::*;
use api::*;

pub struct Display {
    handle: GlWindow,
    events: EventsLoop,
    pub closing: bool,
    pub api: Box<GfxApi>,
}

impl Display {
    pub fn new(width: usize, height: usize) -> Display {
        log!("Init new window with size {}x{}", width, height);
        let events = EventsLoop::new();

        let window = WindowBuilder::new()
            .with_title(format!(
                "{} v{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            )).with_dimensions(glutin::dpi::LogicalSize::new(width as f64, height as f64));

        let context = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (2, 0)))
            .with_vsync(false);

        let gl_window = glutin::GlWindow::new(window, context, &events).unwrap();

        let (width, height) = {
            let size = gl_window.get_inner_size().unwrap();
            (size.width as _, size.height as _)
        };

        log!("Window created");
        log!("GFX API: {:?}", gl_window.get_api());
        log!("GFX res: {}x{}", width, height);
        log!(
            "GFX bit: {}",
            gl_window.context().get_pixel_format().color_bits
        );

        unsafe {
            gl_window.make_current().unwrap();
        }

        //TODO: Move this to api.rs
        let mut count = 0;
        gl::load_with(|symbol| {
            count += 1;
            gl_window.get_proc_address(symbol) as *const _
        });
        log!("Loaded {} GL functions", count);

        Display {
            handle: gl_window,
            events,
            closing: false,
            api: Box::new(GLApi::new(width, height)),
        }
    }

    pub fn swap(&mut self) {
        if let Err(why) = self.handle.swap_buffers() {
            log!("Buffer swap failed: {}", why);
            //TODO: oof
            let size = self
                .handle
                .get_inner_size()
                .unwrap_or(LogicalSize::new(1280.0, 720.0));

            let window = WindowBuilder::new()
                .with_title(format!(
                    "{} v{}",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION")
                )).with_dimensions(glutin::dpi::LogicalSize::new(size.width, size.height));

            let context = glutin::ContextBuilder::new()
                .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (2, 0))); //TODO: Remove

            self.handle = glutin::GlWindow::new(window, context, &self.events).unwrap();

            unsafe {
                self.handle.make_current().unwrap();
            }
            log!("Recreated window");
        };
    }

    pub fn events(&mut self) {

        let mut closing = false;
        let mut resizing = None;

        self.events.poll_events(|e| match e {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => closing = true,
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // TODO: handle resize gracefully
                log!("Window resized to: {}x{}", size.width, size.height);
                resizing = Some((size.width as usize, size.height as usize));
            }
            _ => (),
        });

        self.closing = closing;
        if let Some((width, height)) = resizing {

        self.api.resize(width, height);
        }
    }
}
