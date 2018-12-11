#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![feature(core_intrinsics)]

extern crate panic_abort;
extern crate kea;
extern crate game;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub const fn null<T>() -> *mut T { 0 as *mut T }

struct Api;

impl kea::PlatformApi for Api {
    fn print(&self, s: &str) {
        unsafe { 
            printf(s.as_ptr() as *const i8);
            consoleUpdate(null());
        }
    }
}

#[no_mangle]
pub extern "C" fn main() {
    unsafe { consoleInit(null()); }
    kea::run(Api, &game::game);
    unsafe { consoleExit(null()); }
}

pub mod lang_items;
