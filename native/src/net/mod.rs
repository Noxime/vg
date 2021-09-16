mod certs;
mod client;
mod server;
mod event;

type Error = Box<dyn std::error::Error>;

pub use client::Client;
pub use server::Server;

use self::event::EventHistory;

pub enum S2C {
    Events {
        events: EventHistory,
    }
}