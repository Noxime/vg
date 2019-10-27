use vg::audio;

pub struct Sound;

impl audio::Sound for Sound {
    fn playing(&self) -> bool {
        false
    }

    fn play(&mut self) {
    }

    fn pause(&mut self) {
    }

    fn repeating(&self) -> bool {
        false
    }

    fn set_repeating(&mut self, repeating: bool) {
    }
}

pub struct Audio;

impl audio::Audio for Audio {
    type Sound = Sound;

    fn ogg(&mut self, bytes: Vec<u8>) -> Self::Sound {
        Sound
    }

    fn output(&self) -> Option<audio::Output> {
        None
    }

    fn outputs(&self) -> Vec<audio::Output> {
        vec![]
    }

    fn set_output(&mut self, output: &audio::Output) -> Result<(), String> {
        Ok(())
    }

    fn volume(&self) -> f32 {
        1.0
    }
    fn set_volume(&mut self, volume: f32) {
        
    }

    fn pan(&self) -> f32 {
        0.0
    }
    fn set_pan(&self, pan: f32) {
    }
}

impl Audio {
    pub fn new() -> Self {
        Audio
    }
}
