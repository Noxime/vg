pub type Color = [f32; 4];

pub trait Renderer {
    /// A user friendly name of our rendering engine
    const NAME: &'static str;

    /// Window frame, our final render target
    type Frame: Frame;

    /// Create a new frame object to render to
    fn frame(&mut self, base: Color) -> Self::Frame;
}

pub trait Frame {
    /// Present this frame to the window
    fn present(self, vsync: bool);
}