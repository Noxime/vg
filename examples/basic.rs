#![feature(arbitrary_self_types)]

use vg::*;

#[derive(Serialize, Deserialize)]
struct MonkeyGame {
    time: f32,
    monky: Model,
    ferris: Sprite,
}

impl Game for MonkeyGame {
    fn update(self: &mut Vg<Self>) {
        self.time += self.delta_time().as_secs_f32();
        self.monky.transform.position.x = (self.time * 2.0).sin();

        self.monky.enabled = self
            .player_id(0)
            .map(|id| self.key(id, Key::Space).up())
            .unwrap_or(true);
    }
}

fn main() {
    emoji_logger::init();
    Vg::run(MonkeyGame {
        time: 1.0,
        monky: "suzanne.obj".into(),
        ferris: "ferris.png".into(),
    })
}
