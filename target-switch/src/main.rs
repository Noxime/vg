

extern crate libnx_rs;

extern crate vg;
extern crate game;

use vg::{PlatformApi};

mod renderer;

struct SwitchApi;
impl PlatformApi for SwitchApi {
    fn print(&self, s: &str) { println!("{}", s); }
}

pub fn main() {
    vg::run(SwitchApi, renderer::SwitchRenderer::new(), &game::game);
}