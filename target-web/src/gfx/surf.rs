use std::rc::Rc;
use std::sync::Mutex;
use vg::renderer::{Color, Shading, Size, Transform, View};

use super::Ctx;

pub struct Surf {
    pub ctx: Rc<Ctx>,
    pub waker: Rc<Mutex<Option<std::task::Waker>>>,
    pub matrix: super::webgl::WebGLUniformLocation,
    pub texture: super::webgl::WebGLUniformLocation,
    pub add: super::webgl::WebGLUniformLocation,
    pub multiply: super::webgl::WebGLUniformLocation,
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

impl vg::renderer::Surface<super::Gfx> for Surf {
    fn capture(&self) -> super::Tex {
        unimplemented!()
    }

    fn present(&mut self, vsync: bool) -> Box<dyn std::future::Future<Output = ()> + Unpin> {
        Box::new(Present(Rc::clone(&self.waker), true))
    }
}

impl vg::renderer::Target<super::Gfx> for Surf {
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
        let size = self.size();
        self.ctx.viewport(0, 0, size[0] as _, size[1] as _);

        let (ax, ay) = match view.scale {
            vg::renderer::Scale::Vertical(s) => (self.size()[0] as f32 / self.size()[1] as f32 * s, s),
            vg::renderer::Scale::Horizontal(s) => (s, self.size()[1] as f32 / self.size()[0] as f32 * s),
        };

        let r = transform.rotation - view.rotation;
        let sx = transform.scale_x;
        let sy = transform.scale_y;
        let x = transform.x - view.x;
        let y = transform.y - view.y;

        let tx = texture.size()[0] as f32 / view.pixels_per_unit;
        let ty = texture.size()[1] as f32 / view.pixels_per_unit;

        let vsx = sx / ax * 2.0;
        let vsy = sy / ay * 2.0;

        let matrix: [f32; 16] = [
            r.cos() * vsx * tx, r.sin() * vsy * ty, 0.0, 0.0,
            -r.sin() * vsx * tx, r.cos() * vsy * ty, 0.0, 0.0,
            0.0f32, 0.0, 1.0, 0.0,
            x * vsx, y * vsy, 0.0, 1.0
        ];

        self.ctx.uniform_matrix4fv(Some(&self.matrix), false, &matrix[..]);
        self.ctx.bind_texture(Ctx::TEXTURE_2D, Some(&texture.tex));
        self.ctx.uniform1i(Some(&self.texture), 0);
        self.ctx.uniform4f(Some(&self.add), shading.add[0], shading.add[1], shading.add[2], shading.add[3]);
        self.ctx.uniform4f(Some(&self.multiply), shading.multiply[0], shading.multiply[1], shading.multiply[2], shading.multiply[3]);

        self.ctx.draw_arrays(Ctx::TRIANGLES, 0, 6);
        stdweb::console!(log, "draw");
    }
}
