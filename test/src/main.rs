use vg::*;

fn main() {
    async fn __vg_wrapper() {
        loop {
            gfx::draw("ferris.png").at([0.2, 0.1]);
            gfx::draw("gopher.png").at([100, 200]);

            frame().await;
        }
    }

    __vg_start(__vg_wrapper)
}
