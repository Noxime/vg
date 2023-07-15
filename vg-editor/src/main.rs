use anyhow::Result;
use clap::Parser;
use macroquad::prelude::*;
use std::{f32::consts::E, path::PathBuf, time::Duration};
use tokio::runtime::Runtime;
use tracing_subscriber::EnvFilter;
use vg_interface::{Request, Response};
use vg_network::{Host, HostConfig};
use vg_runtime::executor::DefaultExecutor;

#[derive(Parser)]
struct Args {
    /// Path to WebAssembly module
    path: PathBuf,
    /// URL of signaling server
    #[arg(default_value = "ws://vg.noxim.xyz:3536/")]
    url: String,
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
    let config = HostConfig {
        url: args.url,
        ..Default::default()
    };

    tokio::try_join!(run(&wasm, config.clone()), run(&wasm, config.clone()))?;

    Ok(())
}

async fn run(wasm: &[u8], config: HostConfig) -> Result<()> {
    let func = |_: Request| Response::Empty;
    let (mut host, driver) = Host::<DefaultExecutor>::new(&wasm, func, config)?;

    tokio::spawn(async {
        match driver.await {
            Ok(()) => debug!("Driver closed"),
            Err(err) => error!("Driver error: {err}"),
        }
    });

    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    loop {
        host.tick();
        ticker.tick().await;
    }
}
