use vg::input;

/// Input handler implementation that always returns `None` or an empty `Vec`
pub struct Input;

impl vg::Input for Input {
    fn default(&self) -> Option<input::Id> {
        None
    }

    fn all_controllers(&self) -> Vec<input::Id> {
        vec![]
    }

    fn controller(&self, id: &input::Id) -> Option<input::Controller> {
        None
    }

    fn pointers(&self) -> Vec<input::Pointer> {
        vec![]
    }

    fn mapping(&self) -> input::KeyboardMapping {
        input::KeyboardMapping::default()
    }

    fn set_mapping(&mut self, _: input::KeyboardMapping) {
        
    }
}