#![no_std]

pub mod platform_api;
pub use self::platform_api::PlatformApi;

pub struct EngineApi<Platform: PlatformApi> {
    pub platform: Platform,
}

pub fn run<Platform: PlatformApi>(
    platform: Platform,
    game: &Fn(EngineApi<Platform>),
) {
    let engine = EngineApi {
        platform
    };
    game(engine)
}
