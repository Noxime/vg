extern crate kea;
extern crate game;

struct Api;
impl kea::PlatformApi for Api {
    fn print(&self, s: &str) {
        println!("{}", s);
    }
}

fn main() {  
    kea::run(Api, &game::game);
}
