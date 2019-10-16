mod emscripten;

// The default allocator is quite big so this will make release binaries
// smaller as size is a proper issue on the web
#[cfg_attr(not(debug_assertions), global_allocator)]
#[cfg(not(debug_assertions))]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

async fn game(mut renderer: kea_sdl2::Renderer) {
    use kea::renderer::{Surface, Target, Renderer};
    println!("async loop start");
    loop {
        
        // renderer.poll();
        println!("Loop");
        // renderer.surface().set(&[0.65, 0.87, 0.91, 1.0]);
        renderer.surface().present(true).await;
    }
}

pub fn main() {
    // console_error_panic_hook::set_once();
    println!("Running Kea engine on WASM");

    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

    let kea = kea_sdl2::Renderer::new();

    let fut = &mut game(kea);
    let mut fut = unsafe { Pin::new_unchecked(fut) };

    fn wake(_: *const ())     { println!("wake") }
    fn wake_ref(_: *const ()) { println!("wake_ref") }
    fn drop(_: *const ())     { println!("drop") }
    fn clone(ptr: *const ()) -> RawWaker {
        println!("clone");
        RawWaker::new(
            ptr,
            &RawWakerVTable::new(clone, wake, wake_ref, drop),
        )
    }

    let waker = unsafe { Waker::from_raw(clone(&() as *const ())) };
    let mut context = Context::from_waker(&waker);

    let main_loop = move || {
        // println!("Polling");
        match fut.as_mut().poll(&mut context) {
            Poll::Pending => println!("Expected"),
            Poll::Ready(_) => {
                println!("Game returned");
                unsafe { emscripten::emscripten::emscripten_cancel_main_loop() }
                println!("Exiting");
            },
        }
    };

    println!("Starting main loop");
    emscripten::emscripten::set_main_loop_callback(main_loop);
    println!("Engine exit (the bad way? oh god)");
}
