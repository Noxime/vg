mod emscripten;

// The default allocator is quite big so this will make release binaries
// smaller as size is a proper issue on the web
#[cfg_attr(not(debug_assertions), global_allocator)]
#[cfg(not(debug_assertions))]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

async fn game(mut renderer: kea_sdl2::Renderer) {
    use kea::renderer::{Renderer, Surface, Target};
    println!("async loop start");
    loop {
        // renderer.poll();
        println!("Loop");
        // renderer.surface().set(&[0.65, 0.87, 0.91, 1.0]);
        // renderer.surface().present(true).await;
    }
}

pub fn main() {
    // console_error_panic_hook::set_once();
    println!("Running Kea engine on WASM");

    let kea = kea_sdl2::Renderer::new();

    use futures::future::{FutureExt, TryFutureExt};
    use futures::task::LocalSpawn;
    let mut executor = futures::executor::LocalPool::new();

    async fn foo() {}

    executor
        .spawner()
        .spawn_local_obj(Box::new(async {}).into())
        .expect("Failed to spawn");

    let main_loop = || {
        println!("Loop");
        executor.run_until_stalled();
    };

    println!("Starting main loop");
    emscripten::emscripten::set_main_loop_callback(main_loop);
    println!("Engine exit (the bad way? oh god)");
}
