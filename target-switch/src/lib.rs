#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![feature(core_intrinsics)]

extern crate panic_abort;
// extern crate kea;
// extern crate game;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub const fn null<T>() -> *mut T { 0 as *mut T }

#[no_mangle]
pub extern "C" fn main() {
  unsafe {
    consoleInit(null());

    let mut k_held_old = HidControllerKeys(0);
    
    printf("\x1b[1;1HKEA Engine on switch. Press PLUS to exit\n".as_ptr() as *const i8);
    printf("\x1b[2;1H\n".as_ptr() as *const i8);

    while appletMainLoop() {
      hidScanInput();
      let k_held = HidControllerKeys(hidKeysHeld(HidControllerID::CONTROLLER_P1_AUTO) as u32);

      if k_held == HidControllerKeys::KEY_PLUS {
        break;
      }

      if k_held != k_held_old {
        consoleClear();

        printf("\x1b[1;1HKEA Engine on switch. Press PLUS to exit\n".as_ptr() as *const i8);
        printf("\x1b[2;1H\n".as_ptr() as *const i8);
        printf("\x1b[3;1Hkeycode: %x\n".as_ptr() as *const i8, k_held);
      }

      k_held_old = k_held;

      consoleUpdate(null());
    }

    consoleExit(null());
  }
}

pub mod lang_items;
