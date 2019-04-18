use kea::renderer::{Color, Size, Surface, Target, Texture};
use audio::Clip;
use kea::*;

use std::time::Instant;

const ASSETS: assets::Assets = asset_pack!("assets.keapack");

pub fn game<P, R, I, A>(mut api: EngineApi<P, R, I, A>)
where
    P: PlatformApi,
    R: renderer::Renderer,
    I: input::Input,
    A: audio::Audio,
{
    api.platform.print(&format!("assets.pack contains {} bytes of data", ASSETS.size()));

    let mut clip = api.audio.from_vorbis(ASSETS.assets("audio").unwrap().binary("audio.ogg").unwrap());
    clip.play();

    while !api.platform.exit() {
        api.poll();
        api.renderer.surface().set(&[0.65, 0.87, 0.91, 1.0]);
        api.renderer.surface().present(true);
    }
}
