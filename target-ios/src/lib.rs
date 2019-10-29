extern crate game;
extern crate vg;
extern crate vg_dev;

struct Api;
impl vg::PlatformApi for Api {
    fn print(&self, s: &str) {
        println!("{}", s);
    }
}

#[no_mangle]
pub extern "C" fn ios_main() {
    vg::run(Api, vg_dev::Renderer::new(), &game::game);
}
