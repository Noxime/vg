use std::rc::Rc;
use kea::renderer::{Size, Color, View, Transform, Shading};

pub struct Surf {
    pub ctx: Rc<super::Ctx>,
}

struct Present(usize);
impl std::future::Future for Present {
    type Output = ();

    fn poll(mut self: std::pin::Pin<&mut Present>, ctx: &mut std::task::Context<'_>) 
        -> std::task::Poll<()>
    {
        if self.0 == 0 {
            stdweb::console!(log, "ready");
            std::task::Poll::Ready(())
        } else {
            stdweb::console!(log, "pending");
            self.0 -= 1;
            std::task::Poll::Pending
        }
    }
}

impl kea::renderer::Surface<super::Gfx> for Surf {
    fn capture(&self) -> super::Tex {
        unimplemented!()
    }

    fn present(&mut self, vsync: bool)  -> Box<dyn std::future::Future<Output=()> + Unpin> {
        Box::new(Present(2))
    }
}

impl kea::renderer::Target<super::Gfx> for Surf {
    fn size(&self) -> Size {
        unimplemented!()
    }

    fn set(&mut self, color: &Color) {
        self.ctx.clear_color(color[0], color[1], color[2], color[3]);
        self.ctx.clear(super::Ctx::COLOR_BUFFER_BIT);
    }

    fn draw(&mut self, texture: &super::Tex, shading: &Shading, view: &View, transform: &Transform) {
        unimplemented!()
    }
}