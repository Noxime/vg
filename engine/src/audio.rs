use std::sync::Arc;

#[derive(Debug)]
pub struct Output {
    pub name: String,
    pub channels: usize,
    pub samples: usize,
    pub bits: usize,
}

pub struct Error;

pub trait Audio {
    fn ogg(&mut self, bytes: Vec<u8>) -> Sound;

    // fn create(&mut self) -> Self::Sound;

    fn output(&self) -> Option<Output>;
    fn outputs(&self) -> Vec<Output>;
    fn set_output(&mut self, Output: impl AsRef<Output>) -> Result<(), ()>; // TODO: Errors

    fn volume(&self) -> f32;
    fn set_volume(&mut self, volume: f32);

    fn pan(&self) -> f32;
    fn set_pan(&self, pan: f32);
}

pub struct Sound {
    pub playing: bool,
}

impl Sound {
    pub fn play(&mut self) {
        self.playing = true;
    }
    pub fn pause(&mut self) {
        self.playing = false;
    }
}