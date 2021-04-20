use vg::*;

game! {{
    loop {
        print_str("hello world 😳");
        present().await;
    }
}}
