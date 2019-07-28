use kea::input;
use gilrs;
use std::sync::{Mutex, Arc};

pub struct Input {
    g: gilrs::Gilrs,
}

impl Input {
    pub fn new() -> Self {
        Self {
            g: gilrs::Gilrs::new().expect("Couldn't init gilrs")
        }
    }
}

impl kea::Input for Input {
    fn default(&self) -> Option<input::Id> {
        unimplemented!()
    }

    fn all_controllers(&self) -> Vec<input::Id> {
        unimplemented!()
    }

    fn controller(&self, id: &input::Id) -> Option<input::Controller> {
        unimplemented!()
    }

    fn pointers(&self) -> Vec<input::Pointer> {
        unimplemented!()
    }

    fn mapping(&self) -> input::KeyboardMapping {
        unimplemented!()
    }

    fn set_mapping(&mut self, _: input::KeyboardMapping) {
        unimplemented!()
    }
}
