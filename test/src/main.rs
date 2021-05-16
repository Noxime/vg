use vg::*;

game! {
    loop {
        gfx::draw("ferris.png").pos([time().sin(), time().cos()]);

        frame().await;
    }
}
