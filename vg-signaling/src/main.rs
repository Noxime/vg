mod state;
mod topology;

use std::net::IpAddr;

use anyhow::Result;
use axum::{routing::get, Json};
use clap::Parser;
use matchbox_signaling::SignalingServerBuilder;
use tracing::{debug, error, info, warn, Level};

use crate::{state::VgState, topology::VgTopology};

#[derive(Parser)]
struct Args {
    /// Interface to bind to
    #[arg(long, default_value = "0.0.0.0")]
    host: IpAddr,
    /// Port to serve on
    #[arg(long, default_value = "3536")]
    port: u16,
    /// Logging verbosity
    #[arg(long, default_value = "Info")]
    verbose: Level,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbose)
        .init();

    info!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let topology = VgTopology::new();
    let state = VgState::new();
    let request_state = state.clone();
    let upgrade_state = state.clone();
    let status_state = state.clone();

    let server = SignalingServerBuilder::new((args.host, args.port), topology, state)
        .on_connection_request(move |ws| {
            debug!(addr = ?ws.origin, path = ?ws.path, params = ?ws.query_params, "WebSocket connection request");

            // Save the room key for this socket addr
            request_state.request(ws.origin, ws.path.unwrap_or_default());

            Ok(true)
        })
        .on_id_assignment(move |(addr, peer)| {
            debug!(addr = ?addr, peer = ?peer, "Peer ID assigned");

            // Map socket address to a peer ID
            if !upgrade_state.upgrade(addr, peer) {
                warn!(addr = ?addr, peer = ?peer, "Key was not found");
            }
        })
        .mutate_router(move |router| {
            let state = status_state.clone();
            router.route("/status", get(move || async {
                let state = state;
                Json(state.status())
            }))
        })
        .cors()
        .trace()
        .build();

    if let Err(err) = server.serve().await {
        error!("Server error: {err}");
        std::process::exit(1);
    }

    Ok(())
}
