use wgpu::Texture;

use super::Head;

impl Head {
    /// Render 2D content
    pub fn render_canvas(&mut self, _surface: &Texture) {
        // self.canvas
        //     .clear_rect(10, 10, 50, 50, femtovg::Color::rgbf(1.0, 0.5, 1.0));

        // self.canvas.renderer_mut().set_screen(&surface);
        // self.canvas.flush();
    }
}
