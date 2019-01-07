extern crate game;
extern crate kea;
extern crate kea_dev;

struct Api;
impl kea::PlatformApi for Api {
    fn print(&self, s: &str) {
        println!("{}", s);
    }
}

fn main() {
    kea::run(Api, kea_dev::Renderer::new(), &game::game);
}
