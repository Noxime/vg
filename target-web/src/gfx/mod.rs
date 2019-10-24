use std::rc::Rc;
use std::sync::Mutex;
use std::task::Waker;

use webgl_stdweb as webgl;
pub use webgl::WebGLRenderingContext as Ctx;

use stdweb::{console, js};

mod surf;
mod tex;
use surf::Surf;
use tex::Tex;

pub struct Gfx {
    surface: Surf
}

impl Gfx {
    pub fn new(canvas: stdweb::web::html_element::CanvasElement) -> (Gfx, Rc<Mutex<Option<std::task::Waker>>>) {
        let ctx: Ctx = canvas.get_context().unwrap();

        ctx.clear_color(1.0, 0.7, 1.0, 1.0);
        ctx.clear(Ctx::COLOR_BUFFER_BIT);

        console!(log, "WebGL Context created; ", ctx.get_context_attributes());

        let waker = Rc::new(Mutex::new(None));

        (Gfx {
            surface: Surf {
                ctx: Rc::new(ctx),
                waker: Rc::clone(&waker),
            }
        }, waker)
    }
}

impl kea::Renderer for Gfx {
    const NAME: &'static str = "WebGL";
    type Texture = Tex;
    type Surface = Surf;

    fn surface(&mut self) -> &mut Surf {
        &mut self.surface
    }
}