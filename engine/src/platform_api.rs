pub trait PlatformApi {
    /// Should the game exit?
    fn exit(&self) -> bool;
    /// Print some debug text into the console
    fn print(&self, _: &str);
}