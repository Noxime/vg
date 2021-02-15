#![feature(arbitrary_self_types)]

use vg::*;

#[derive(Serialize, Deserialize)]
struct MonkeyGame {
    buddha: Model,
}

#[derive(Serialize, Deserialize)]
struct MonkeyPlayer {
    ferris: Sprite,
}

impl Player<MonkeyGame> for MonkeyPlayer {
    fn connected(_: &mut Vg<MonkeyGame>, _: PlayerId) -> Self {
        Self {
            ferris: "ferris.png".into()
        }
    }
}

impl Game for MonkeyGame {
    type Player = MonkeyPlayer;

    fn update(self: &mut Vg<Self>) {
        self.buddha.transform.position.y = self.run_time().as_secs_f32().sin().signum();

        let speed = self.delta_time().as_secs_f32();

        for p in self.players_mut() {
            let movement = p.wasd_arrows() * speed;
            p.ferris.transform.position += movement;
        }
    }
}

fn main() {
    emoji_logger::init();
    vg::run(MonkeyGame {
        buddha: "suzanne.obj".into(),
    })
}
