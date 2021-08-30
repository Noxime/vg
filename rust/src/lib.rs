// pub use vg_derive::game;
// use wasm_bindgen::prelude::*;

use std::{collections::VecDeque, future::Future, time::Duration};
mod conversions;
pub use conversions::{Position, Rotation};
use vg_types::DeBin;
pub mod gfx;
mod input;
pub mod sfx;
pub use input::*;

// If we are running in WASI we are inside VG and everything is okay.
#[cfg(target_os = "wasi")]
fn ensure() -> &'static mut State {
    unsafe {
        STATE.get_or_insert_with(|| {
            let tick = 0;
            let responses = VecDeque::new();

            State {
                tick,
                responses,
                runtime: Duration::from_secs(0),
                deltatime: Duration::from_secs(0),
                input: Input::default(),
            }
        })
    }
}

// This is what happens when you don't use cargo-vg
#[cfg(not(target_os = "wasi"))]
fn ensure() -> &'static mut State {
    let mut code = Some(vg_builder::WASM.to_vec());
    vg_native::Engine::run::<vg_native::runtime::wasmer::Wasmer, _>(move || code.take())
}

// WASM-local engine state
static mut STATE: Option<State> = None;

pub struct State {
    // Current tick number. Monotonically increasing at a fixed pace (deltatime)
    tick: usize,
    // The messages delivered from hosting VG, to be decoded
    responses: VecDeque<Vec<u8>>,
    // Time since the start of game aka tick 0
    runtime: Duration,
    // How long a single tick takes
    deltatime: Duration,
    // Input state cache
    input: Input,
}

#[link(wasm_import_module = "env")]
extern "C" {
    // Provide a pointer length to a bincoded Call to the hosting VG instance
    fn call(ptr: u64, len: u64);
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

pub fn frame() {
    call_host(vg_types::Call::Present);

    let state = ensure();
    state.tick += 1;
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
