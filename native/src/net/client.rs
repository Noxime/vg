use futures::StreamExt;
use quinn::{
    Connection, Datagrams, EndpointBuilder, IncomingBiStreams, IncomingUniStreams, NewConnection,
};
use tokio::{net::{lookup_host, ToSocketAddrs}, select};
use tracing::*;

use crate::{debug::Memory, net::certs, runtime::Runtime};

use super::Error;

pub struct Client<RT> {
    connection: Connection,
    uni_streams: IncomingUniStreams,
    bi_streams: IncomingBiStreams,
    datagrams: Datagrams,
    state: ClientState<RT>,
}

enum ClientState<RT> {
    Game { runtime: [RT; 0] },
}

impl<RT: Runtime> Client<RT> {
    pub async fn connect(addr: impl ToSocketAddrs) -> Result<Self, Error> {
        let remote = lookup_host(addr).await?.next().unwrap();

        let cfg = certs::insecure_client();
        let endpoint = EndpointBuilder::new(Default::default(), cfg);

        let (endpoint, _) = endpoint.bind(&"[::]:0".parse().unwrap())?;
        let connecting = endpoint.connect(&remote, "localhost")?;
        debug!("Connecting to {}", connecting.remote_address());
        let NewConnection {
            connection,
            datagrams,
            uni_streams,
            bi_streams,
            ..
        } = connecting.await?;
        debug!("Connected, ID: {}", connection.stable_id());

        Ok(Client {
            connection,
            bi_streams,
            uni_streams,
            datagrams,
            state: ClientState::Game { runtime: [] },
        })
    }

    pub async fn run(mut self) -> Result<(), Error> {
        let mut runtime = None;

        loop {
            select! {
                Some(datagram) = self.datagrams.next() => {
                    debug!("Got datagram: {:?}", datagram);

                    let _bytes = datagram?;
                },
                Some(uni) = self.uni_streams.next() => {
                    debug!("Got unidirectional stream");
                    let stream = uni?;
                    let bytes = stream.read_to_end(4 * crate::debug::GB).await?;
                    debug!("Received {} bytes of state", Memory(bytes.len()));

                    runtime = Some(RT::load(&bytes));
                },
                _bi = self.bi_streams.next() => {
                    debug!("Got bidirectional stream");
                }
            }
        }
    }
}
