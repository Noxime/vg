use std::{
    convert::Infallible,
    net::{IpAddr, SocketAddr},
};

use anyhow::Result;
use derivative::Derivative;
use hyper::{
    body::{to_bytes, HttpBody},
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Client, Request, Response, Server,
};
use tokio::{
    net::UdpSocket,
    sync::{broadcast, mpsc},
};
use tracing::{debug, error, info};

use crate::net::HTTP_PORT;

use super::server::FullTick;

#[derive(Clone)]
struct Context {
    addr_tx: mpsc::Sender<UdpSocket>,
    tick_tx: broadcast::Sender<FullTick>,
}

pub async fn run(
    addr: IpAddr,
    addr_tx: mpsc::Sender<UdpSocket>,
    tick_tx: broadcast::Sender<FullTick>,
) -> Result<()> {
    let ctx = Context { addr_tx, tick_tx };

    let service = make_service_fn(|conn: &AddrStream| {
        let ctx = ctx.clone();

        let addr = conn.remote_addr();

        let service = service_fn(move |req| handle(ctx.clone(), addr, req));

        async move { Ok::<_, Infallible>(service) }
    });

    let server = Server::bind(&(addr, HTTP_PORT).into()).serve(service);
    info!("HTTP server on {:?}", addr);

    if let Err(err) = server.await {
        error!("HTTP server closed: {}", err);
    }

    Ok(())
}

// POST /<port> -> <port>
async fn handle(
    ctx: Context,
    mut client_addr: SocketAddr,
    req: Request<Body>,
) -> Result<Response<Body>> {
    debug!("Allocating a socket for client");
    let port = req
        .headers()
        .get("VG-Client-Port")
        .unwrap()
        .to_str()?
        .parse()?;
    client_addr.set_port(port);

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(client_addr).await?;
    let server_addr = socket.local_addr()?;
    debug!(
        "Client UDP: {:?}, server UDP: {:?}",
        client_addr, server_addr
    );

    ctx.addr_tx.send(socket).await?;

    let mut rx = ctx.tick_tx.subscribe();
    let FullTick { state, .. } = rx.recv().await?;

    let resp = Response::builder()
        .header("VG-Server-Port", server_addr.port())
        .body(Body::from(state))?;

    Ok(resp)
}

// Request an allocated socket from the host
pub async fn req_socket(host_addr: IpAddr) -> Result<(UdpSocket, Vec<u8>)> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let client_addr = socket.local_addr()?;

    let url = format!("http://{}:{}/sync", host_addr, HTTP_PORT);

    let client = Client::new();
    let req = Request::get(url)
        .header("VG-Client-Port", client_addr.port())
        .body(Body::empty())?;

    let mut res = client.request(req).await?;

    let port: u16 = res
        .headers()
        .get("VG-Server-Port")
        .unwrap()
        .to_str()?
        .parse()?;

    // let bytes = to_bytes(res).await?;
    // let string = String::from_utf8(bytes.to_vec())?;
    // let port: u16 = string.parse()?;

    let host_addr = SocketAddr::from((host_addr, port));
    debug!(
        "Requested UDP socket, client: {:?}, host: {:?}",
        client_addr, host_addr
    );

    socket.connect(host_addr).await?;
    let state = to_bytes(res.body_mut()).await?.to_vec();

    Ok((socket, state))
}
