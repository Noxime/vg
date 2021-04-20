pub mod runtime;

use runtime::Runtime;

pub fn run<RT: Runtime>(code: &[u8]) -> ! {
    println!("Starting VG");

    let mut runtime = RT::load(code).expect("Loading the runtime failed");
    let mut engine = Engine::new();

    loop {
        runtime.run_tick(&mut engine).unwrap();
    }
}

pub struct Engine {}

impl Engine {
    fn new() -> Engine {
        Engine {}
    }
}
