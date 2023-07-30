use winit::event_loop::EventLoopBuilder;
use winit::platform::android::{activity::AndroidApp, EventLoopBuilderExtAndroid};

// Entry point for android applications
#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Trace),
    );

    let event_loop = EventLoopBuilder::new().with_android_app(app).build();

    let mut engine = crate::Engine::new();

    // TODO: Asset loading for Android

    event_loop.run(move |event, target, control_flow| {
        engine.event(&event, target);

        if engine.alive() {
            control_flow.set_poll();
        } else {
            control_flow.set_exit();
        }
    })
}

pub fn main() {
    unreachable!("Should not be called on android");
}
