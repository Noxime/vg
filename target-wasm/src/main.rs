#![feature(set_stdio)]

use webgl_stdweb as webgl;
use stdweb::{console, js};
use stdweb::web::{self, html_element::CanvasElement, IParentNode, INode};
use stdweb::unstable::TryInto;

// The default allocator is quite big so this will make release binaries
// smaller as size is a proper issue on the web
#[cfg_attr(not(debug_assertions), global_allocator)]
#[cfg(not(debug_assertions))]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

async fn game(mut api: impl kea::Renderer) {
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
    console!(log, "Kea start");

    let document = web::document();
    let element = document.create_element("canvas").unwrap();
    let canvas: CanvasElement = element.try_into().unwrap();
    let body = document.body().unwrap();
    body.append_child(&canvas);
    canvas.set_width(800);
    canvas.set_height(600);

    console!(log, "Canvas created");

    use webgl::WebGLRenderingContext as Ctx;
    let ctx: webgl::WebGLRenderingContext = canvas.get_context().unwrap();

    console!(log, "WebGL Error: ", ctx.get_error());

    ctx.clear_color(1.0, 0.7, 1.0, 1.0);
    ctx.clear(Ctx::COLOR_BUFFER_BIT);

    console!(log, "Context: ", ctx.get_context_attributes());

    // let kea = kea_sdl2::Renderer::new();

    use futures::future::{FutureExt, TryFutureExt};
    use futures::task::LocalSpawn;
    let mut executor = futures::executor::LocalPool::new();

    executor
        .spawner()
        .spawn_local_obj(Box::new(async {}).into())
        .expect("Failed to spawn");

    let main_loop = || {
        println!("Loop");
    };

    fn draw(ctx: Ctx, mut executor: futures::executor::LocalPool, x: f32) {
        console!(log, "updating");
        executor.run_until_stalled();

        ctx.clear_color(x.sin() * 0.5 + 0.5, 0.5, 0.5, 1.0);
        ctx.clear(Ctx::COLOR_BUFFER_BIT);

        web::window().request_animation_frame(move |_| draw(ctx, executor, x + 1.0/60.0));
    }

    draw(ctx, executor, 0.0);

    println!("Engine exit (the bad way? oh god)");
}
