use std::ops::{Deref, Range};

use nanoserde::{DeBin, SerBin};
use vg_types::{Event, PlayerEvent};

#[derive(SerBin, DeBin, Clone, Debug)]
pub struct EventQueue {
    seq: u64,
    events: Vec<Event>,
}

impl EventQueue {
    pub fn new() -> EventQueue {
        EventQueue {
            seq: 0,
            events: vec![],
        }
    }

    pub fn push(&mut self, ev: Event) {
        self.events.push(ev);
    }

    // Clear events up to i seq
    pub fn ack(&mut self, i: u64) {
        let rem = i.saturating_sub(self.seq) as usize;
        self.events.drain(0..rem.min(self.events.len()));
        self.seq += rem as u64;
    }

    pub fn up_to(&self) -> u64 {
        self.events.len() as u64 + self.seq
    }

    pub fn range(&self) -> Range<u64> {
        self.seq..self.up_to()
    }

    pub fn empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn after(&self, i: u64) -> &[Event] {
        let rem = i.saturating_sub(self.seq);
        if self.range().contains(&rem) {
            &self.events[rem as usize..]
        } else {
            &[]
        }
    }
}
