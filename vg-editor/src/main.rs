use clap::Parser;
use macroquad::prelude::*;
use std::path::PathBuf;
use vg_interface::{Draw, Request, Response};
use vg_network::Server;
use vg_runtime::executor::DefaultExecutor;

#[derive(Parser)]
struct Args {
    /// Path to WebAssembly module
    path: PathBuf,
}

#[macroquad::main("vg")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let wasm = std::fs::read(args.path)?;

    let func = |request: Request| match request {
        Request::Draw(Draw::Line {
            color: (r, g, b, a),
            points,
        }) => {
            for win in points.windows(2) {
                draw_line(
                    win[0].0,
                    win[0].1,
                    win[1].0,
                    win[1].1,
                    1.0,
                    Color { r, g, b, a },
                );
            }
            Response::Empty
        }
    };

    let mut server = Server::<_, DefaultExecutor<_>>::start(&wasm, true, func)?;

    loop {
        clear_background(BLACK);

        server.tick();

        next_frame().await;
    }
}
