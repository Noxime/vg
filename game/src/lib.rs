use kea::renderer::{Surface, Target};
// use audio::Clip;
use kea::*;

const ASSETS: assets::Assets = asset_pack!("assets.keapack");

pub fn run<A: Api>(mut api: A) {
    println!("assets.pack contains {} bytes of data", ASSETS.size());

    // let mut clip = api.audio().from_vorbis(ASSETS.assets("audio").unwrap().binary("audio.ogg").unwrap());
    // clip.play();


    while !api.exit() {
        api.poll();
        let id = api.input().default().unwrap();
        if api.input().controller(&id).unwrap().start.active() {
            // clip.stop();
            // clip.play();
        }

        api.renderer().surface().set(&[0.65, 0.87, 0.91, 1.0]);
        api.renderer().surface().present(true);
    }
}
