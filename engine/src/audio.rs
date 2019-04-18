//! # Audio abstraction
//! 
//! THIS FILE IS UNDER CONSTRUCTION AND **WILL** CHANGE IN THE NEXT COMMIT

#[derive(PartialEq)]
pub enum Kind {
    Mono,
    Stereo,
}

/// An audio file, which you can load and [`start`](Audio::start)
pub trait Audio {
    type Clip: Clip;
    /// Load the audio from vorbis data
    fn from_vorbis(&self, bytes: &[u8]) -> Self::Clip;
}

pub trait Clip {
    /// Start playing the audio clip, stopping when reaching the end
    fn play(&mut self);
    /// Pause the audio clip
    fn pause(&mut self);
    /// Pause and rewind to beginning of the track
    fn stop(&mut self) {
        self.pause();
        self.seek(0.0);
    }
    /// Sets this [`Clip`] to loop over and over
    fn repeat(&mut self, repeat: bool);
    /// The total length of this audio clip, in seconds
    fn length(&self) -> f32;
    /// Seek to a second
    /// 
    /// Note: Panics if seeking beyond the length of this audio clip
    fn seek(&mut self, time: f32);
    /// The current time of the playing clip in seconds
    fn time(&self) -> f32;
    /// Is the clip done playing?
    fn done(&self) -> bool;
}