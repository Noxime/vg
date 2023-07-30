use winit::event_loop::EventLoopBuilder;

#[allow(dead_code)] // Called from main.rs
pub fn main() {
    tokio::runtime::Runtime::new()
        .expect("Failed to start executor")
        .block_on(async {
            let event_loop = EventLoopBuilder::new().build();

            let mut engine = crate::Engine::new();

            // TODO: Disable hot reloading for release builds
            tokio::spawn(vg_asset::FileSource::run(engine.assets().clone(), "."));

            event_loop.run(move |event, target, control_flow| {
                engine.event(&event, target);

                if engine.alive() {
                    control_flow.set_poll();
                } else {
                    control_flow.set_exit();
                }
            });
        });
}
