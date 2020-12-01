#![feature(arbitrary_self_types)]

use vg::*;

#[derive(Serialize, Deserialize)]
struct MyGame {
    tick_number: usize,
    monky: Model,
}

impl Game for MyGame {
    fn update(self: &mut Vg<Self>) {

        self.tick_number += 1;
    }
}

fn main() {
    emoji_logger::init();
    Vg::run(MyGame {
        tick_number: 0,
        monky: "suzanne.obj".into()
    })
}
