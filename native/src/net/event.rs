use std::time::Duration;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Event {
    Tick {
        tickrate: Duration,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventHistory {
    seq: usize,
    events: Vec<Event>,
}

impl EventHistory {
    pub fn new() -> EventHistory {
        EventHistory {
            seq: 0,
            events: vec![]
        }
    }

    pub fn since(&self, seq: usize) -> &[Event] {
        assert!(seq >= self.seq, "Tried to get {} while we are at {}", seq, self.seq);

        let i = seq - self.seq;
        &self.events[i..]
    }
}