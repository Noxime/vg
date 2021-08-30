use vg::*;

fn main() {
    let mut pos = [0.0; 2];

    loop {
        let keys = wasd();

        pos[0] += keys[0] * delta() as f32;
        pos[1] += keys[1] * delta() as f32;

        if Key::Space.pressed() {
            sfx::play("cat.ogg");
        }

        gfx::draw("ferris.png").pos(pos);
        gfx::draw("ferris.png").pos([1, 1]);

        frame();
    }
}
