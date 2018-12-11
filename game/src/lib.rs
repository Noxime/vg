extern crate kea;

use kea::*;

pub fn game(mut api: EngineApi<impl PlatformApi, impl renderer::Renderer>) {
    api.platform.print("Hello world");
    let mut f: f32 = 0.0;
    loop {
        f += 0.4;
        use kea::renderer::*;
        api.renderer
            .frame([f.sin().signum() * 0.5 + 0.5, f.cos().signum() * 0.5 + 0.5, 0.5, 1.0])
            .present(true);
    }
}