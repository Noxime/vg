pub mod platform_api;
pub mod renderer;
pub use self::platform_api::PlatformApi;

pub struct EngineApi<Platform: PlatformApi, Renderer: renderer::Renderer> {
    pub platform: Platform,
    pub renderer: Renderer,
}

pub fn run<Platform: PlatformApi, Renderer: renderer::Renderer>(
    platform: Platform,
    renderer: Renderer,
    game: &Fn(EngineApi<Platform, Renderer>),
) {
    let engine = EngineApi {
        platform,
        renderer,
    };
    engine.platform.print("Running Kea");
    game(engine);
}
