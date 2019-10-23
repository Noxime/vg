#![feature(set_stdio)]

use webgl_stdweb as webgl;
use stdweb::{console, js};
use stdweb::web::{self, html_element::CanvasElement, IParentNode, INode};
use stdweb::unstable::TryInto;

mod gfx;

// The default allocator is quite big so this will make release binaries
// smaller as size is a proper issue on the web
#[cfg_attr(not(debug_assertions), global_allocator)]
#[cfg(not(debug_assertions))]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

async fn game(mut api: impl kea::Renderer) {
    use kea::renderer::{Renderer, Surface, Target};
    println!("async loop start");
    let mut t = 0.0f32;
    loop {
        t += 1.0/120.0;
        console!(log, "Loop");
        api.surface().set(&[t.sin() * 0.5 + 0.5, 0.87, 0.91, 1.0]);
        api.surface().present(true).await;
    }
}

pub fn main() {
    // console_error_panic_hook::set_once();
    console!(log, "Kea start");

    let document = web::document();
    let element = document.create_element("canvas").unwrap();
    let canvas: CanvasElement = element.try_into().unwrap();
    let body = document.body().unwrap();
    body.append_child(&canvas);
    canvas.set_width(800);
    canvas.set_height(600);

    let kea = gfx::Gfx::new(canvas);

    use futures::future::{FutureExt, TryFutureExt};
    use futures::task::LocalSpawn;
    use futures::executor::LocalPool;
    let executor = LocalPool::new();

    executor
        .spawner()
        .spawn_local_obj(Box::new(game(kea)).into())
        .expect("Failed to spawn");

    fn main_loop(mut executor: LocalPool) {
        executor.run_until_stalled();
        console!(log, "Done with executor");
        web::window().request_animation_frame(move |_| main_loop(executor));
    }

    main_loop(executor);

    console!(log, "Engine exit (the bad way? oh god)");
}
