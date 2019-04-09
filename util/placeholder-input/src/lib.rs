/// Input handler implementation that always returns `None` or an empty `Vec`
pub struct Input;

impl kea::Input for Input {
    fn default(&self) -> Option<kea::input::Id> {
        None
    }

    fn all_controllers(&self) -> Vec<kea::input::Id> {
        vec![]
    }

    fn controller(&self, _: &kea::input::Id) -> Option<kea::input::Controller> {
        None
    }
}