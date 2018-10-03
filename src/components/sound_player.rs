use audio::*;
use components::Component;
use graphics::*;

pub struct SoundPlayer {
    path: String,
}

impl SoundPlayer {
    pub fn new(path: &str) -> SoundPlayer { SoundPlayer { path: path.into() } }
    pub fn play(&self) {
        play(self.path.clone());
    }
}

impl Component for SoundPlayer {}

