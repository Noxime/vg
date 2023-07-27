use winit::event_loop::EventLoopBuilder;

#[allow(dead_code)] // Called from main.rs
pub fn main() {
    let event_loop = EventLoopBuilder::with_user_event().build();

    let mut engine = crate::Engine::new();
    event_loop.run(move |event, _, control_flow| {
        // Run as fast as possible
        control_flow.set_poll();
        engine.event(event);
    });
}
