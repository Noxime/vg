use vg::*;

main!(my_game);

async fn my_game() {
    println!("Hello world");

    let mut time = 0.0f32;

    loop {
        let color = V(0.0, 0.5, 0.2, 1.0);

        let start = V(20.0, 20.0);
        let mid = V(30.0 + time.sin() * 15.0, 30.0 + time.cos() * 15.0);
        let end = V(40.0, 60.0);

        line(color, [start, mid, end]);

        time += 0.1;

        present().await;
    }
}
