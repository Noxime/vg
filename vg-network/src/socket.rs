use matchbox_socket::{
    MessageLoopFuture, MultipleChannels, WebRtcChannel, WebRtcSocket, WebRtcSocketBuilder,
};
use tracing::debug;

pub struct Socket {
    socket: WebRtcSocket<MultipleChannels>,
    reliable: WebRtcChannel,
    unreliable: WebRtcChannel,
}

impl Socket {
    pub fn new(url: &str) -> (Socket, MessageLoopFuture) {
        let (mut socket, driver) = WebRtcSocketBuilder::new(url)
            .reconnect_attempts(None)
            .add_reliable_channel()
            .add_unreliable_channel()
            .build();

        (
            Socket {
                reliable: socket.take_channel(0).unwrap(),
                unreliable: socket.take_channel(1).unwrap(),
                socket,
            },
            driver,
        )
    }

    pub fn poll(&mut self) {
        for (peer, state) in self.socket.update_peers() {
            debug!(?peer, ?state, "Peer state change");
        }
        for _ in self.reliable.receive() {}
        for _ in self.unreliable.receive() {}
    }
}
