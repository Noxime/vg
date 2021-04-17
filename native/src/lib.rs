pub mod runtime;

use runtime::Runtime;

pub fn run<RT: Runtime>(code: &[u8]) -> ! {
    println!("Starting VG");

    let mut runtime = RT::load(code).expect("Loading the runtime failed");
    let mut engine = Engine::new();

    println!("Original");
    for _ in 0..3 {
        runtime.run_tick(&mut engine).unwrap();
    }

    let mut restore = runtime.duplicate().unwrap();

    println!("Saved rollback");
    for _ in 0..3 {
        runtime.run_tick(&mut engine).unwrap();
    }

    println!("Rollback");
    for _ in 0..3 {
        restore.run_tick(&mut engine).unwrap();
    }

    std::process::exit(0)
}

pub struct Engine {}

impl Engine {
    fn new() -> Engine {
        Engine {}
    }
}
