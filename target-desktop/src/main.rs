extern crate game;
extern crate kea;
extern crate kea_piston_renderer;

struct Api;
impl kea::PlatformApi for Api {
    fn print(&self, s: &str) {
        println!("{}", s);
    }
}

fn main() {
    kea::run(Api, kea_piston_renderer::PistonRenderer::new(), &game::game);
}
