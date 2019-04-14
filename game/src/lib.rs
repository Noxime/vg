use kea::renderer::{Color, Matrix, Size, Surface, Target, Texture};
use kea::*;

use std::time::Instant;

const ASSETS: assets::Assets = asset_pack!("assets.keapack");

pub fn game<P, R, I>(mut api: EngineApi<P, R, I>)
where
    P: PlatformApi,
    R: renderer::Renderer,
    I: input::Input,
{
    api.platform.print(&format!("assets.pack contains {} bytes of data", ASSETS.size()));

    while !api.platform.exit() {
        api.poll();
        api.renderer.surface().set(&[0.65, 0.87, 0.91, 1.0]);
        api.renderer.surface().present(true);
    }
}
