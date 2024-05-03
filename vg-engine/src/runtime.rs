use vg_interface::{Draw, Request, Response};
use vg_runtime::{executor::Instance, Provider};

use crate::prelude::*;
use crate::Engine;

#[profile_all]
impl Engine {
    /// Run the instance until a new frame is ready
    pub(crate) fn run_frame(&mut self) -> Nil {
        // Done before check to keep asset loading active
        let instance = Check::from(self.instance.get())?;

        // If runtime is paused, don't advance
        Check::from(self.config.running)?;

        // Reset immediate state
        self.world.reset_frame();

        // Run until frame is ready
        while !instance.step(&mut self.world).is_present() {}

        Nil
    }
}

#[derive(Default, Clone)]
pub struct WorldState {
    pub draws: Vec<Draw>,
}

impl WorldState {
    fn reset_frame(&mut self) {
        self.draws.clear();
    }
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
