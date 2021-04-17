use vg::*;

game! {
    for i in 0.. {
        foo(i);
        present().await;
    }
}
