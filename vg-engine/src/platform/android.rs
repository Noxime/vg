use winit::event_loop::EventLoopBuilder;
use winit::platform::android::{activity::AndroidApp, EventLoopBuilderExtAndroid};

// Entry point for android applications
#[no_mangle]
fn android_main(app: AndroidApp) {
    let executor = tokio::runtime::Runtime::new().expect("Failed to start executor");
    let _guard = executor.enter();

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Trace),
    );

    let event_loop = EventLoopBuilder::new()
        .with_android_app(app)
        .build()
        .unwrap();

    let mut engine = crate::Engine::new();

    // TODO: Asset loading for Android

    engine.run_winit(event_loop);
}
