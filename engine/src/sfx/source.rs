mod sine;
pub use sine::SineSource;

pub trait Source {
    /// Fill the buffer with some samples from this source, and return false
    /// if done playing
    ///
    /// # Note
    /// The buffer is always interlaced stereo, and samples should be in range
    /// `-1.0 .. 1.0`
    fn sample(&mut self, buffer: &mut [f32], sample_rate: usize) -> bool;
}
