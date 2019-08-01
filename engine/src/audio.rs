#[derive(Debug)]
pub struct Output {
    pub name: String,
    pub channels: usize,
    pub samples: usize,
    pub bits: usize,
}

impl AsRef<Output> for Output {
    fn as_ref(&self) -> &Self {
        self
    }
}

pub struct Error;

pub trait Audio {
    type Sound: Sound;
    fn ogg(&mut self, bytes: Vec<u8>) -> Self::Sound;

    // fn create(&mut self) -> Self::Sound;

    fn output(&self) -> Option<Output>;
    fn outputs(&self) -> Vec<Output>;
    fn set_output(&mut self, output: &Output) -> Result<(), String>; // TODO: Errors

    fn volume(&self) -> f32;
    fn set_volume(&mut self, volume: f32);

    fn pan(&self) -> f32;
    fn set_pan(&self, pan: f32);
}

pub trait Sound {
    fn playing(&self) -> bool;
    fn play(&mut self);
    fn pause(&mut self);

    fn repeating(&self) -> bool;
    fn set_repeating(&mut self, repeating: bool);
}
