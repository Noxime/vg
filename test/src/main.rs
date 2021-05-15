use vg::*;

game! {
    let mut time = 0.0f32;

    loop {
        print(time);
        time += 0.1;

        gfx::draw("ferris.png").at([time.sin(), time.cos()]);

        print("I am after draw");

        frame().await;
    }
}
