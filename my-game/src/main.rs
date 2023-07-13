use vg::*;

main!(my_game);

async fn my_game() {
    println!("Hello world");

    loop {
        line(WHITE, [Vec2::new(20.0, 20.0), Vec2::new(40.0, 40.0)]);

        present().await;
    }

}
