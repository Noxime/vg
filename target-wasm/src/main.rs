mod emscripten;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// #[wasm_bindgen]
pub fn main() {
    console_error_panic_hook::set_once();
    println!("Running Kea engine on WASM");

    let main_loop = || {};

    // game::run(Wasm {});

    emscripten::emscripten::set_main_loop_callback(main_loop);
    println!("Engine exit (the bad way? oh god)");
}
