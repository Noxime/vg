use winit::event_loop::EventLoopBuilder;
use winit::platform::android::{activity::AndroidApp, EventLoopBuilderExtAndroid};

// Entry point for android applications
#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Trace),
    );

    let event_loop = EventLoopBuilder::with_user_event()
        .with_android_app(app)
        .build();

    let mut engine = crate::Engine::new();
    event_loop.run(move |event, _, control_flow| {
        // Run as fast as possible
        control_flow.set_poll();
        engine.event(event);
    })
}

pub fn main() {
    unreachable!("Should not be called on android");
}
