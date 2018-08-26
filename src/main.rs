#[macro_use]
mod macros;
mod api;
mod arch;
mod display;
mod shader;
mod sprite;

use display::Display;

fn main() {
    // initialize architecture specific systems
    arch::init();
    log!("Arch initialized");
    let mut display = Display::new(1280, 720);

    let shader = match display.api.compile_shader(
        //&arch::load_string("shaders/default.vs").expect("shaders/default.vs not found"),
        //&arch::load_string("shaders/default.fs").expect("shaders/default.fs not found"),
        include_str!("../assets/shaders/default.vs"),
        include_str!("../assets/shaders/default.fs")
    ) {
        Ok(v) => v,
        Err(shader::ShaderError::CompileError(why)) => panic!("Shader compilation failed: {}", why),
        _ => panic!("Shader compilation failed"),
    };

    // log!("Loading default sprite");
    // let sprite = sprite::Sprite::new("textures/test.png");

    use std::time::Instant;
    let mut start = Instant::now();
    let mut frames = 0;

    while !display.closing {
        if start.elapsed().as_secs() > 0 {
            log!("FPS: {} ({:.2}ms)", frames, 1000.0 / frames as f32);
            frames = 0;
            start = Instant::now();
        } else {
            frames += 1;
        }

        display.events();

        // rendering code
        display.api.clear(1.0, 1.0, 1.0);

        // present
        display.swap();
    }
}
