//! Audio

mod audio;
mod source;
pub use audio::Audio;
pub use source::{SineSource, Source};

/// Audio API
pub trait Sfx {
    /// Create a playable audio clip from an audio source
    ///
    /// # Example
    /// ```rust
    /// let audio = vg.audio(SinceSource::new(440.0));
    ///
    /// // play a 440 hz sine wave at 20% volume
    /// audio.volume(0.2);
    /// audio.play();
    /// ```
    fn audio(&self, source: Box<dyn Source>) -> Audio;
}
