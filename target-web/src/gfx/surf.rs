use kea::renderer::{Color, Shading, Size, Transform, View};
use std::rc::Rc;
use std::sync::Mutex;

use super::Ctx;

pub struct Surf {
    pub ctx: Rc<Ctx>,
    pub waker: Rc<Mutex<Option<std::task::Waker>>>,
}

struct Present(Rc<Mutex<Option<std::task::Waker>>>, bool);
impl std::future::Future for Present {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Present>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<()> {
        let mut set = self.1;
        if let Ok(mut mutex) = self.0.lock() {
            // the waker has been taken, either means first poll or consaumed
            if mutex.is_none() {
                // first poll
                if self.1 {
                    *mutex = Some(ctx.waker().clone());
                    set = false;
                } else {
                    // it got taken, so we have yielded
                    return std::task::Poll::Ready(());
                }
            }
        } else {
            panic!("Failed to lock mutex")
        }
        self.1 = set;

        std::task::Poll::Pending
    }
}

impl kea::renderer::Surface<super::Gfx> for Surf {
    fn capture(&self) -> super::Tex {
        unimplemented!()
    }

    fn present(&mut self, vsync: bool) -> Box<dyn std::future::Future<Output = ()> + Unpin> {
        Box::new(Present(Rc::clone(&self.waker), true))
    }
}

impl kea::renderer::Target<super::Gfx> for Surf {
    fn size(&self) -> Size {
        [
            self.ctx.drawing_buffer_width() as usize,
            self.ctx.drawing_buffer_height() as usize,
        ]
    }

    fn set(&mut self, color: &Color) {
        self.ctx.clear_color(color[0], color[1], color[2], color[3]);
        self.ctx.clear(super::Ctx::COLOR_BUFFER_BIT);
    }

    fn draw(
        &mut self,
        texture: &super::Tex,
        shading: &Shading,
        view: &View,
        transform: &Transform,
    ) {
        // let size = self.size();
        // self.ctx.viewport(0, 0, size[0] as _, size[1] as _);
        self.ctx.draw_arrays(Ctx::TRIANGLES, 0, 6);
        stdweb::console!(log, "draw");
    }
}
