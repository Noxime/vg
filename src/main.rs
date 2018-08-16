#[macro_use]
mod macros;
mod api;
mod arch;
mod display;

use display::Display;

fn main() {
    // initialize architecture specific systems
    arch::init();
    log!("Arch initialized");
    let mut display = Display::new(1280, 720);

    // let shader = match display.api.compile_shader(
    //     &arch::load_string("shaders/default.vs").expect("shaders/default.vs not found"),
    //     &arch::load_string("shaders/default.fs").expect("shaders/default.fs not found"),
    // ) {
    //     Ok(v) => v,
    //     Err(api::ShaderError::CompileError(why)) => panic!("Shader compilation failed: {}", why),
    //     _ => panic!("Shader compilation failed"),
    // };

    use std::time::Instant;
    let time = Instant::now();

    while !display.closing {
        display.events();
        display.swap(time.elapsed().subsec_millis() as f32 / 1000.0);
    }
}
