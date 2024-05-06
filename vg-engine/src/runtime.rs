use std::fmt::Display;

use vg_interface::{Draw, Request, Response};
use vg_runtime::{
    executor::{Instance, InstanceData},
    Provider,
};

use crate::prelude::*;
use crate::Engine;

/// Represents a point in "time" for the game
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct RuntimeInstant {
    frame: usize,
}

impl RuntimeInstant {
    pub const EPOCH: RuntimeInstant = RuntimeInstant { frame: 0 };

    /// Calculate number of frames since some earlier instant
    pub fn frames_since(self, before: RuntimeInstant) -> isize {
        self.frame as isize - before.frame as isize
    }

    pub fn prev_frame(mut self) -> RuntimeInstant {
        self.frame -= 1;
        self
    }

    pub fn next_frame(mut self) -> RuntimeInstant {
        self.frame += 1;
        self
    }
}

impl Display for RuntimeInstant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}f", self.frame)
    }
}

#[profile_all]
impl Engine {
    /// Run the instance until a new frame is ready
    pub(crate) fn run_tick(&mut self) -> Check {
        // Done before check to keep asset loading active
        let instance = Check::from(self.instance.get())?;

        // Record things
        let mut world = WorldState::default();

        // Run until frame is ready
        while !instance.step(&mut world).is_present() {}
        self.instant.frame += 1;

        // Update the presentation world
        self.world = world;
        self.redraw();

        PASS
    }

    /// Produce a save state from the current state, which can be used to restore
    pub fn save_state(&mut self) -> Option<SaveState> {
        let instance = self.instance.get()?;

        let data = instance.get_data();

        Some(SaveState {
            data,
            instant: self.instant,
        })
    }

    /// Set the instance state to some premade save state
    pub fn restore_state(&mut self, save_state: &SaveState) -> Result<()> {
        let instance = self.instance.get().ok_or(anyhow!("What"))?;

        instance.set_data(&save_state.data);
        self.instant = save_state.instant;

        Ok(())
    }

    pub fn runtime_instant(&self) -> RuntimeInstant {
        self.instant
    }
}

pub struct SaveState {
    data: InstanceData,
    instant: RuntimeInstant,
}

impl SaveState {
    pub fn instant(&self) -> RuntimeInstant {
        self.instant
    }
}

#[derive(Default, Clone)]
pub struct WorldState {
    pub draws: Vec<Draw>,
}

#[profile_all]
impl Provider for WorldState {
    fn provide(&mut self, request: Request) -> Response {
        match request {
            Request::Draw(draw) => self.draws.push(draw),
        }

        Response::Empty
    }
}
