pub mod platform_api;
pub mod renderer;
pub use self::platform_api::PlatformApi;

pub struct EngineApi<Platform: PlatformApi, Renderer: renderer::Renderer> {
    pub platform: Platform,
    pub renderer: Renderer,
}

pub struct Transform {
    pos: (f32, f32),
    sca: (f32, f32),
    rot: f32,
}

impl Default for Transform {
    fn default() -> Transform {
        Transform {
            pos: (0.0, 0.0),
            sca: (1.0, 1.0),
            rot: 0.0,
        }
    }
}

pub struct Sprite {
    trans: Transform,
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
