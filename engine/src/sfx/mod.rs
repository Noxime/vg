//! Audio

mod audio;
mod source;
pub use audio::Audio;
pub use source::{SineSource, Source};

pub struct Sfx {
    sfx: Box<dyn SfxTrait>,
}

impl Sfx {
    #[cfg_attr(not(feature = "dev-docs"), doc(hidden))]
    pub fn new(sfx: Box<dyn SfxTrait>) -> Self {
        Self {
            sfx
        }
    }
    
    pub fn audio(&self, source: impl Source + 'static) -> Audio {
        self.sfx.audio(Box::new(source))
    }
}

/// Audio API
pub trait SfxTrait {
    /// Create a playable audio clip from an audio source
    ///
    /// # Example
    /// ```rust
    /// let audio = vg.audio(SineSource::new(440.0));
    ///
    /// // play a 440 hz sine wave at 20% volume
    /// audio.volume(0.2);
    /// audio.play();
    /// ```
    fn audio(&self, source: Box<dyn Source>) -> Audio;
}
