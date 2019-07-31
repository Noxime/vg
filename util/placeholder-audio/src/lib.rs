use kea::audio;

pub struct Sound;

impl audio::Sound for Sound {
    fn playing(&self) -> bool {
        unimplemented!()
    }

    fn play(&mut self) {
        unimplemented!()
    }

    fn pause(&mut self) {
        unimplemented!()
    }

    fn repeating(&self) -> bool {
        unimplemented!()
    }

    fn set_repeating(&mut self, repeating: bool) {
        unimplemented!()
    }
}

pub struct Audio;

impl audio::Audio for Audio {
    type Sound = Sound;

    fn ogg(&mut self, bytes: Vec<u8>) -> Self::Sound {
        unimplemented!()
    }

    fn output(&self) -> Option<audio::Output> {
        unimplemented!()
    }

    fn outputs(&self) -> Vec<audio::Output> {
        unimplemented!()
    }

    fn set_output(&mut self, output: &audio::Output) -> Result<(), String> {
        unimplemented!()
    }

    fn volume(&self) -> f32 {
        unimplemented!()
    }
    fn set_volume(&mut self, volume: f32) {
        unimplemented!()
    }

    fn pan(&self) -> f32 {
        unimplemented!()
    }
    fn set_pan(&self, pan: f32) {
        unimplemented!()
    }
}

impl Audio {
    pub fn new() -> Self {
        Audio
    }
}
