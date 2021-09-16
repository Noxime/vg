use std::time::Duration;

use futures::StreamExt;
use quinn::{CertificateChain, Endpoint, EndpointBuilder, Incoming, NewConnection, ServerConfigBuilder};
use tokio::{net::{ToSocketAddrs, lookup_host}, time::sleep};
use tracing::*;

use crate::{net::certs, runtime::Runtime};

use super::Error;

pub struct Server<RT> {
    endpoint: Endpoint,
    incoming: Incoming,
    runtime: RT,
    // state: ServerState<RT>,
}

// enum ServerState<RT> {
//     Game { runtime: RT },
// }

impl<RT: Runtime> Server<RT> {
    pub async fn bind(addr: impl ToSocketAddrs, code: Vec<u8>) -> Result<Self, Error> {
        let remote = lookup_host(addr).await?.next().unwrap();

        let mut endpoint = EndpointBuilder::new(Default::default(), Default::default());

        let (cert, pk) = certs::generate_self_signed_cert(vec!["localhost".into()])?;

        let mut cfg = ServerConfigBuilder::default();
        cfg.certificate(CertificateChain::from_certs(vec![cert]), pk)?;
        endpoint.listen(cfg.build());

        debug!("Listening on {}", remote);
        let (endpoint, incoming) = endpoint.bind(&remote)?;

        let runtime = RT::load(&code)?;

        Ok(Server {
            endpoint,
            incoming,
            runtime,
        })
    }

    pub async fn run(mut self) -> Result<(), Error> {
        while let Some(connecting) = self.incoming.next().await {
            debug!("Accepting connection from {}", connecting.remote_address());
            let NewConnection { connection, .. } = connecting.await?;
            debug!("Connected, ID: {}", connection.stable_id());

            // Sync runtime state with client
            let bytes = self.runtime.serialize()?;
            let mut stream = connection.open_uni().await?;
            stream.write_all(&bytes).await?;
            stream.finish().await?;
            
            loop {
                connection.send_datagram(b"Hello world"[..].into())?;
                sleep(Duration::from_secs(1)).await;
                debug!("{:?}, max size: {:?}", connection.rtt(), connection.max_datagram_size());
            }
        }

        debug!("Server exit");

        Ok(())
    }
}
