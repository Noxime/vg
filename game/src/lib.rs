extern crate kea;

use kea::*;
use kea::renderer::Target;

pub fn game<P, R>(mut api: EngineApi<P, R>)
where
    P: PlatformApi,
    R: renderer::Renderer,
{
    {
        let [w, h] = api.renderer.surface().size();
        api.platform.print(&format!("Renderer is: {}", R::NAME));
        api.platform.print(&format!("Window is: {}x{}", w, h));
    }

    let mut f: f32 = 0.0;

    loop {
        f += 0.05;

        api.renderer.surface().set(&[f.sin(), f.cos(), 0.5, 1.0]);
    }
}