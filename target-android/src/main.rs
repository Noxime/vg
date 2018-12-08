extern crate kea;
extern crate game;

fn main() {
    kea::run(&mut game::loader, game::INITIAL);
}
