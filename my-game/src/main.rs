use vg::*;

main!(my_game);

async fn my_game() {
    println!("Hello world");

    loop {
        let color = V(1.0, 0.5, 0.2, 1.0);
        let start = V(20.0, 20.0);
        let end = V(40.0, 60.0);

        line(color, [start, end]);

        present().await;
    }
}
