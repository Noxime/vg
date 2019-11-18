//! Audio

mod source;
mod audio;
pub use audio::Audio;
pub use source::{Source, SineSource};

/// Audio API
pub trait Sfx {
    fn audio(&self, source: Box<dyn Source>) -> Audio;
}