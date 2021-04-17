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
pub static mut STATE: Option<State> = None;

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
    unreachable!("We should be in WASM now")
}

pub struct State {
    tick: usize,
    exec: executor::Executor,
}

#[link(wasm_import_module = "env")]
extern "C" {
    fn print(_: i32);
}

pub fn __vg_start<F, Fut>(f: F)
where
    F: Fn() -> Fut,
    Fut: Future<Output = ()> + 'static,
{
    ensure();

    // let exec = executor::Executor::new(f());
    // let tick = 0;

    // unsafe { STATE = Some(State { exec, tick }) };
}

#[no_mangle]
pub extern "C" fn __vg_tick() {
    let state = ensure();
    state.tick += 1;
    // let State { exec, .. } = unsafe { STATE.as_mut() }.unwrap();

    // exec.run();
    foo(state.tick as _)
    // rt.run_until(async { rx.next().await });
}

pub fn foo(s: i32) {
    ensure();
    #[cfg(target_arch = "wasm32")]
    unsafe {
        print(s);
    }
}

pub async fn present() {
    println!("Present");
    ensure().exec.halt().await;
}
