mod emscripten;
use wasm_bindgen::prelude::*;

// // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// // allocator.
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

// #[wasm_bindgen]
pub fn main() {
    // console_error_panic_hook::set_once();
    println!("Running Kea engine on WASM");

    // use glium::Surface;
    // use glium_sdl2::DisplayBuild;
    use sdl2;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    println!("sub done");

    let display = video_subsystem
        .window("Kea", 800, 600)
        .resizable()
        // .opengl()
        // .build_glium()
        .build()
        .unwrap();

    let mut canvas = display.into_canvas().build().unwrap();

    println!("display done");

    let mut running = true;
    let mut event_pump = sdl_context.event_pump().unwrap();

    println!("pump done");
    let mut main_loop = || {
        // let mut target = display.draw();
        // target.clear_color(0.5, 0.2, 0.8, 1.0);
        // target.finish().expect("Present failed");

        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 255, 255));
        canvas.clear();
        canvas.present();
        println!("present");
        // Event loop: polls for events sent to all windows

        for event in event_pump.poll_iter() {
            use sdl2::event::Event;

            match event {
                Event::Quit { .. } => {
                    std::process::exit(0);
                }
                _ => (),
            }
        }
    };

    emscripten::emscripten::set_main_loop_callback(main_loop);

    println!("Engine exit");
}
