use vg::*;

#[game(assets = "examples/assets/")]
async fn main(mut vg: Vg) {
    vg.title("VG - Sprite");

    let ferris = asset::png(vg.asset("ferris.png").unwrap());
    let tex = vg.texture(ferris).await;

    loop {
        while let Some(e) = vg.event() {
            if e == Event::Exit {
                return;
            }
        }

        // draw 1 copy of `tex` at the center of the screen
        vg.fill(Color::blue());
        vg.draw(
            &tex,
            &[Mat::identity().scaled_3d(0.2).rotated_z(vg.time() as _)],
        );

        vg.present().await;
    }
}
