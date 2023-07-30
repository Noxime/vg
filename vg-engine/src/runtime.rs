use vg_interface::{Draw, Request, Response};
use vg_runtime::{executor::Instance, Provider};

use crate::Engine;

impl Engine {
    /// Run the instance until a new frame is ready
    pub(crate) fn run_frame(&mut self) {
        // Done before check to keep asset loading active
        let Some(instance) = self.instance.get() else { return };

        if !self.config.running {
            return;
        }

        self.scene.reset_frame();

        // Run until frame is ready
        while !instance.step(&mut self.scene).is_present() {}
    }
}

#[derive(Default, Clone)]
pub struct SceneState {
    pub draws: Vec<Draw>,
}
impl SceneState {
    fn reset_frame(&mut self) {
        self.draws.clear();
    }
}

impl Provider for SceneState {
    fn provide(&mut self, request: Request) -> Response {
        match request {
            Request::Draw(draw) => self.draws.push(draw),
        }

        Response::Empty
    }
}
