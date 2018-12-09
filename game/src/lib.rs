#![no_std]

extern crate kea;
use kea::*;

pub fn game(api: EngineApi<impl PlatformApi>) {
    api.platform.print("Hello world");
    loop {}
}