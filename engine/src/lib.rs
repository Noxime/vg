pub mod platform_api;
pub mod renderer;
pub mod input;
pub use self::platform_api::PlatformApi;
pub use self::renderer::Renderer;
pub use self::input::Input;

pub struct EngineApi<Platform: PlatformApi, Renderer: renderer::Renderer, Input: input::Input> {
    pub platform: Platform,
    pub renderer: Renderer,
    pub input: Input,
    pub poll: Box<FnMut()>,
}

impl<Platform: PlatformApi, Renderer: renderer::Renderer, Input: input::Input> EngineApi<Platform, Renderer, Input> {
    pub fn poll(&mut self) {
        (self.poll)()
    }
}

pub fn run<Platform: PlatformApi, Renderer: renderer::Renderer, Input: input::Input>(
    platform: Platform,
    renderer: Renderer,
    input: Input,
    poll: Box<FnMut()>,
    game: &Fn(EngineApi<Platform, Renderer, Input>),
) {
    let engine = EngineApi {
        platform,
        renderer,
        input,
        poll,
    };
    engine.platform.print("Running Kea");
    game(engine);
}
