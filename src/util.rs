use std::time::{Duration, Instant};

use crate::PlayerEvent;

pub trait IsFuture {
    fn is_future(&self) -> bool;
}

impl IsFuture for Instant {
    fn is_future(&self) -> bool {
        let now = Instant::now();
        !self.saturating_duration_since(now).is_zero()
    }
}

pub trait Lerp {
    fn lerp(&mut self, other: Self, t: f32);
}

impl Lerp for f32 {
    fn lerp(&mut self, other: Self, t: f32) {
        *self = *self * (1.0 - t) + other * t;
    }
}

pub trait TimedEvents {
    fn take_before(&mut self, before: Duration) -> Vec<PlayerEvent>;
    fn tick(&mut self, delta: Duration);
    fn rollback_events(&self, rollback: Duration, tickrate: Duration) -> Vec<Vec<PlayerEvent>>;
}

impl TimedEvents for Vec<(Duration, PlayerEvent)> {
    fn take_before(&mut self, before: Duration) -> Vec<PlayerEvent> {
        self.sort_by_key(|(i, _)| *i);
        self.drain_filter(|(ago, _)| *ago >= before)
            .map(|(_, event)| event)
            .collect()
    }

    fn tick(&mut self, delta: Duration) {
        for (ago, _) in self {
            *ago += delta;
        }
    }

    fn rollback_events(&self, rollback: Duration, tickrate: Duration) -> Vec<Vec<PlayerEvent>> {
        let mut copy = self.clone();
        let mut res = vec![];

        let rollback_ticks = rollback.div_duration_f32(tickrate).floor() as u32;

        for tick in 0..rollback_ticks {
            res.push(copy.take_before(rollback - tickrate * tick));
        }

        res
    }
}
