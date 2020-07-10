use vg::*;

#[game(assets = "examples/assets/")]
async fn main(mut vg: Vg) {
    vg.title("VG - Audio");

    // let ferris = asset::png(vg.asset("ferris.png").unwrap());
    // let tex = vg.texture(ferris).await;

    // let mut transform = Mat::identity();

    loop {
        while let Some(e) = vg.event() {
            if e == Event::Exit { return }
        }

        
        vg.fill(Color::green());
        vg.present().await;
    }
}
