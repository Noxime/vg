use vg::*;

game!(test);
async fn test() {
    let mut pos = [0.0; 2];

    loop {
        for (p, k) in pos.iter_mut().zip(wasd().iter()) {
            *p += *k * delta() as f32;
        }

        if Key::Space.pressed() {
            sfx::play("cat.ogg");
        }

        gfx::draw("ferris.png").pos(pos);

        frame().await;
    }
}
