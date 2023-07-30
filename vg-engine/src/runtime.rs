use vg_interface::{Request, Response};
use vg_runtime::{executor::Instance, Provider};

use crate::Engine;

impl Engine {
    /// Run the instance until a new frame is ready
    pub(crate) fn run_frame(&mut self) {
        // Done before check to keep asset loading active
        let Some(instance) = self.instance.get() else { return };

        if !self.config.running {
            return
        }

        let mut blah = SceneState {};

        // Run until frame is ready
        while !instance.step(&mut blah).is_present() {}
    }
}

struct SceneState {}

impl Provider for SceneState {
    fn provide(&mut self, _request: Request) -> Response {
        Response::Empty
    }
}
