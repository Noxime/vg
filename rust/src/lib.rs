// pub use vg_derive::game;
// use wasm_bindgen::prelude::*;

use std::{collections::VecDeque, future::Future, time::Duration};
mod conversions;
mod executor;
pub use conversions::{Position, Rotation};
use vg_types::DeBin;
pub mod gfx;
mod input;
pub mod sfx;
pub use input::*;

#[macro_export]
macro_rules! game {
    ($i:ident) => {
        fn main() {
            __vg_start($i)
        }
    };
}

#[cfg(target_os = "wasi")]
fn ensure() -> &'static mut State {
    unsafe { STATE.get_or_insert_with(|| unreachable!()) }
}

// This is what happens when you don't use cargo-vg
#[cfg(not(target_os = "wasi"))]
fn ensure() -> &'static mut State {
    let mut code = Some(vg_builder::WASM.to_vec());
    vg_native::Engine::run::<vg_native::runtime::wasm::Wasm, _>(move || code.take())
}

static mut STATE: Option<State> = None;

pub struct State {
    tick: usize,
    exec: executor::Executor,
    responses: VecDeque<Vec<u8>>,
    runtime: Duration,
    deltatime: Duration,
    input: Input,
}

#[link(wasm_import_module = "env")]
extern "C" {
    fn call(ptr: u64, len: u64);
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
            let responses = VecDeque::new();

            State {
                exec,
                tick,
                responses,
                runtime: Duration::from_secs(0),
                deltatime: Duration::from_secs(0),
                input: Input::default(),
            }
        });
    }

    ensure();
}

#[no_mangle]
pub extern "C" fn __vg_tick() {
    let state = ensure();

    state.tick += 1;
    state.exec.run();
}

/// Give the host some way to allocate new memory in the client
#[no_mangle]
pub extern "C" fn __vg_allocate(len: u64) -> u64 {
    let resp = &mut ensure().responses;

    resp.push_back(vec![0; len as usize]);
    resp.back().unwrap().as_ptr() as u64
}

/// Take a host-pushed Response and apply it to local state
// #[no_mangle]
// pub extern "C" fn __vg_consume() {
//     let state = ensure();

//     while let Some(bytes) = state.responses.pop_front() {
//         match vg_types::Response::deserialize_bin(&bytes).unwrap() {
//             vg_types::Response::Time(step) => {
//                 state.deltatime = Duration::from_secs_f64(step);
//                 state.runtime += state.deltatime;
//             }
//             vg_types::Response::Up(key) => {
//                 state.input.set(key, Digital::Raised)
//             },
//             vg_types::Response::Down(key) => {
//                 state.input.set(key, Digital::Pressed)
//             },
//             vg_types::Response::Tick => {
//                 // state.input.tick();
//             }
//         }
//     }
// }

fn call_host(val: impl vg_types::SerBin) {
    ensure();
    #[cfg(target_os = "wasi")]
    unsafe {
        let bytes = val.serialize_bin();
        call(bytes.as_ptr() as u64, bytes.len() as u64)
    }
}

// Public api

pub fn time() -> f64 {
    ensure().runtime.as_secs_f64()
}

pub fn delta() -> f64 {
    ensure().deltatime.as_secs_f64()
}

pub fn print(s: impl ToString) {
    call_host(vg_types::Call::Print(s.to_string()))
}

pub async fn frame() {
    call_host(vg_types::Call::Present);
    let state = ensure();
    state.exec.halt().await;
    state.input.step_states();

    while let Some(bytes) = state.responses.pop_front() {
        match vg_types::Response::deserialize_bin(&bytes).unwrap() {
            vg_types::Response::Time(step) => {
                state.deltatime = Duration::from_secs_f64(step);
                state.runtime += state.deltatime;
            }
            vg_types::Response::Up(key) => state.input.set(key, Digital::Raised),
            vg_types::Response::Down(key) => state.input.set(key, Digital::Pressed),
            vg_types::Response::Tick => {
                // state.input.tick();
            }
        }
    }
}
