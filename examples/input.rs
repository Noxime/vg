use vg::{input::gamepad::Axis, *};

#[game(assets = "examples/assets/")]
async fn main(mut vg: Vg) {
    vg.title("VG - Sprite");

    let ferris = asset::png(vg.asset("ferris.png").unwrap());
    let tex = vg.texture(ferris).await;

    let mut transform = Mat::identity();

    loop {
        while let Some(e) = vg.event() {
            if e == Event::Exit {
                return;
            }
        }

        let gamepad = vg.input().main();
        let xy = Pos::new(*gamepad.axis(Axis::LeftX), *gamepad.axis(Axis::LeftY));
        transform.translate_2d(xy * 5.0 * vg.delta_time() as f32);

        vg.fill(Color::white());
        vg.draw(&tex, &[transform.scaled_3d(0.2)]);

        vg.present().await;
    }
}
