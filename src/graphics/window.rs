use vectors::*;
use winit::*;

pub struct Window {
    pub events: EventsLoop,
    pub wb: Option<WindowBuilder>,
}

impl Window {
    pub fn new(size: Vec2<usize>, title: &str) -> Window {
        debug!("creating window with size {}x{}", size.x, size.y);
        let events = EventsLoop::new();
        let wb = WindowBuilder::new()
            .with_dimensions(dpi::LogicalSize::new(size.x as _, size.y as _))
            .with_title(title);
        Window {
            events,
            wb: Some(wb),
        }
    }

    pub fn close(self) {
        // TODO: clean
    }
}
