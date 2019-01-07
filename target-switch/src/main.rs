

extern crate libnx_rs;

extern crate kea;
extern crate game;

use kea::{PlatformApi};

mod renderer;

struct SwitchApi;
impl PlatformApi for SwitchApi {
    fn print(&self, s: &str) { println!("{}", s); }
}

pub fn main() {
    kea::run(SwitchApi, renderer::SwitchRenderer::new(), &game::game);
}