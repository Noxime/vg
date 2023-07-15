use anyhow::Result;
use clap::Parser;
use macroquad::prelude::*;
use std::{path::PathBuf, time::Duration};
use tokio::runtime::Runtime;
use tracing_subscriber::EnvFilter;
use vg_interface::{Request, Response};
use vg_network::Host;
use vg_runtime::executor::DefaultExecutor;

#[derive(Parser)]
struct Args {
    /// Path to WebAssembly module
    path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("cranelift_codegen=warn".parse()?)
                .add_directive("wasmtime_cranelift=warn".parse()?)
                .add_directive("wasmtime_jit=warn".parse()?),
        )
        .init();

    let runtime = Runtime::new()?;
    let _runtime_guard = runtime.enter();

    let args = Args::parse();

    let wasm = tokio::fs::read(args.path).await?;

    server(&wasm).await?;
    // tokio::try_join!(server(&wasm), client(&wasm),)?;

    Ok(())
}

async fn server(wasm: &[u8]) -> Result<()> {
    let func = |_: Request| Response::Empty;

    let (mut host, driver) = Host::<DefaultExecutor>::start(&wasm, true, func)?;

    tokio::spawn(async {
        match driver.await {
            Ok(()) => debug!("Server closed"),
            Err(err) => error!("Server error: {err}"),
        }
    });

    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    loop {
        host.tick();
        ticker.tick().await;
    }
}

// async fn client(wasm: &[u8]) -> Result<()> {
//     let (mut host, driver) = Host::<DefaultExecutor>::connect("ws://localhost:3536")?;

//     tokio::spawn(async {
//         match driver.await {
//             Ok(()) => debug!("Client closed"),
//             Err(err) => error!("Client error: {err}"),
//         }
//     });

//     loop {
//         host.tick();
//         tokio::task::yield_now().await;
//     }
// }
