use vg::*;

game! {
    loop {
        gfx::draw("ferris.png").pos([time().sin(), time().cos()]).rot(time());

        print("I am after draw");

        frame().await;
    }
}
