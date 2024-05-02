use winit::event_loop::EventLoopBuilder;

#[allow(dead_code)] // Called from main.rs
pub fn main() {
    let executor = tokio::runtime::Runtime::new().expect("Failed to start executor");
    let _guard = executor.enter();

    let event_loop = EventLoopBuilder::new().build().unwrap();
    let engine = vg_engine::Engine::new();

    // TODO: Disable hot reloading for release builds
    tokio::spawn(vg_asset::FileSource::run(engine.assets().clone(), "."));

    engine.run_winit(event_loop);
}
