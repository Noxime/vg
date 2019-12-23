/// An audio source producing a sine wave
pub struct SineSource(f64, f32);

impl SineSource {
    pub fn new(frequency: f32) -> SineSource {
        SineSource(0.0, frequency)
    }
}

impl super::Source for SineSource {
    fn sample(&mut self, buffer: &mut [f32], rate: usize) -> bool {
        for i in 0..buffer.len() / 2 {
            self.0 += buffer.len() as f64 / 2.0 / rate as f64;
            buffer[i * 2 + 0] = (self.0 * self.1 as f64).sin() as f32;
            buffer[i * 2 + 1] = (self.0 * self.1 as f64).sin() as f32;
        }

        true
    }
}
