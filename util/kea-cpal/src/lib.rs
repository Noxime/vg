use cpal;
use kea::audio;

pub struct Audio {}

impl audio::Audio for Audio {
    fn outputs(&self) -> Vec<audio::Output> { unimplemented!() }
    fn default_output(&self) -> Option<audio::Output> { unimplemented!() }
    fn set_output(&mut self, id: &audio::Id) { unimplemented!() }

    fn volume(&self) -> f32 { unimplemented!() }
    fn set_volume(&mut self, volume: f32) { unimplemented!() }

    fn pan(&self) -> f32 { unimplemented!() }
    fn set_pan(&self, pan: f32) { unimplemented!() }
}

impl Audio {
    pub fn new() -> Self {
        unimplemented!()
    }
}
