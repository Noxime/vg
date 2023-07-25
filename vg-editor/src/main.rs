use anyhow::Result;
use clap::Parser;
use macroquad::prelude::*;
use std::{path::PathBuf, time::Duration};
use tokio::time::Instant;
use tracing_subscriber::EnvFilter;
use vg_interface::WaitReason;
use vg_network::{ClientData, HostData, Role, Socket};
use vg_runtime::executor::{DefaultExecutor, Executor, Instance};

#[derive(Parser)]
struct Args {
    /// Path to WebAssembly module
    path: PathBuf,
    /// URL of signaling server
    #[arg(long, default_value = "ws://vg.noxim.xyz:3536/")]
    url: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("vg-network=trace".parse()?)
                .add_directive("matchbox_socket=error".parse()?)
                .add_directive("cranelift_codegen=warn".parse()?)
                .add_directive("wasmtime_cranelift=warn".parse()?)
                .add_directive("wasmtime_jit=warn".parse()?)
                .add_directive("webrtc_sctp=warn".parse()?),
        )
        .init();

    let args = Args::parse();

    let wasm = tokio::fs::read(args.path).await?;

    tokio::try_join!(run(&wasm, &args.url), run(&wasm, &args.url))?;

    Ok(())
}

async fn run(wasm: &[u8], url: &str) -> Result<()> {
    let (mut socket, driver) = Socket::new(url)?;

    tokio::spawn(async {
        match driver.await {
            Ok(()) => debug!("Driver closed"),
            Err(err) => error!("Driver error: {err}"),
        }
    });

    let func = |_| vg_interface::Response::Empty;
    let instance = DefaultExecutor::create(wasm, true, func)?;

    loop {
        tokio::task::yield_now().await;

        let Some(role) = socket.poll_role() else { continue };

        debug!("Socket role: {role:?}");

        break match role {
            Role::Host => run_host(instance, socket).await,
            Role::Client => run_client(instance, socket).await,
        };
    }
}

async fn run_host(mut instance: impl Instance, mut socket: Socket) -> Result<()> {
    let mut host = HostData::new();
    let mut instant = Instant::now();

    loop {
        tokio::task::yield_now().await;

        host.poll(&mut socket)?;

        // It is time for a server tick
        let elapsed = instant.elapsed();
        if elapsed > Duration::from_millis(10) {
            // Execute one tick
            loop {
                match instance.step() {
                    WaitReason::Startup => continue,
                    WaitReason::Present => break,
                }
            }

            // Announce new tick
            let data = instance.get_data();
            host.tick(&mut socket, &data, elapsed)?;

            instant += elapsed;
        }
    }
}

async fn run_client(mut instance: impl Instance, mut socket: Socket) -> Result<()> {
    let mut client = ClientData::new();

    loop {
        tokio::task::yield_now().await;

        let Some(confirm) = client.poll(&mut socket)? else { continue };

        // Deserialize state if server pushed
        if let Some(data) = confirm.state()? {
            instance.set_data(&data);
        }

        // Execute one tick
        loop {
            match instance.step() {
                WaitReason::Startup => continue,
                WaitReason::Present => break,
            }
        }

        let data = instance.get_data();
        // Desync
        if confirm.diverged(&data) {
            debug!("Client diverged, resyncing");
            client.desync(&mut socket)?;
        }
    }
}
