// pub use vg_derive::game;
// use wasm_bindgen::prelude::*;

use std::future::Future;
mod executor;

#[macro_export]
macro_rules! game {
    ($e: expr) => {
        fn main() {
            async fn __vg_wrapper() {
                $e
            }

            __vg_start(__vg_wrapper)
        }
    };
}

// #[no_mangle]
static mut STATE: Option<State> = None;

#[cfg(target_arch = "wasm32")]
fn ensure() -> &'static mut State {
    unsafe {
        STATE.get_or_insert_with(|| {
            let exec = executor::Executor::new(async {});
            let tick = 0;

            State { exec, tick }
        })
    }
}

// This is what happens when you don't use cargo-vg
#[cfg(not(target_arch = "wasm32"))]
fn ensure() -> &'static mut State {
    const WASM: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/magic-build/out.wasm"));

    let _engine = vg_native::run::<vg_native::runtime::wasm::Wasm>(WASM);
    // run is -> ! so we should be in WASM now
}

pub struct State {
    tick: usize,
    exec: executor::Executor,
}

#[allow(unused)]
#[link(wasm_import_module = "env")]
extern "C" {
    fn print(ptr: u32, len: u32);
}

pub fn __vg_start<F, Fut>(f: F)
where
    F: Fn() -> Fut,
    Fut: Future<Output = ()> + 'static,
{
    unsafe {
        STATE.get_or_insert_with(|| {
            let exec = executor::Executor::new(f());
            let tick = 0;

            State { exec, tick }
        });
    }

    ensure();

    // let exec = executor::Executor::new(f());
    // let tick = 0;

    // unsafe { STATE = Some(State { exec, tick }) };
}

#[no_mangle]
pub extern "C" fn __vg_tick() {
    let state = ensure();
    state.tick += 1;
    // foo(state.tick as _);

    // let State { exec, .. } = unsafe { STATE.as_mut() }.unwrap();
    state.exec.run();

    // exec.run();
    // rt.run_until(async { rx.next().await });
}

#[allow(unused)]
pub fn print_str(s: &str) {
    ensure();
    #[cfg(target_arch = "wasm32")]
    unsafe {
        let ptr = s.as_ptr() as u32;

        print(ptr, s.len() as u32);
    }
}

pub async fn present() {
    println!("Present");
    ensure().exec.halt().await;
}
