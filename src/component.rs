pub trait Component {
    fn initialize(&mut self) {}
    fn destroy(&mut self) {}
}