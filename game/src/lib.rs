use vg::*;
use gfx::Target;

// const ASSETS: assets::Assets = asset_pack!("assets.vgpack");

pub async fn run(mut vg: Vg, mut gfx: Gfx, mut sfx: Sfx) {

    loop {
        while let Some(event) = vg.poll_event() {
            println!("{:?}", event);
            match event {
                Event::Exit => return,
                _ => ()
            }
        }

        gfx.fill(Color::GREEN);
        gfx.present().await;
    }
}
