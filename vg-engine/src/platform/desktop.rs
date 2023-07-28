use winit::event_loop::EventLoopBuilder;

#[allow(dead_code)] // Called from main.rs
pub fn main() {
    let event_loop = EventLoopBuilder::new().build();

    let mut engine = crate::Engine::new();
    event_loop.run(move |event, target, control_flow| {
        engine.event(&event, target);

        if engine.alive() {
            control_flow.set_poll();
        } else {
            control_flow.set_exit();
        }
    });
}
