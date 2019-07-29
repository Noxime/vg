use std::sync::Arc;

#[derive(Debug)]
pub struct Id(usize);

#[derive(Debug)]
pub struct Output {
    id: Id,
    name: String,
    channels: usize,
    bitrate: usize,
    bitdepth: usize,
}

pub struct Error;

pub trait Audio {
    fn ogg(&mut self, bytes: Arc<[u8]>) -> Sound {
        Sound {
            ogg: bytes,
            playing: Arc::new(false),
        }
    }

    fn outputs(&self) -> Vec<Output>;
    fn default_output(&self) -> Option<Output>;
    fn set_output(&mut self, id: &Id);

    fn volume(&self) -> f32;
    fn set_volume(&mut self, volume: f32);

    fn pan(&self) -> f32;
    fn set_pan(&self, pan: f32);
}

pub struct Sound {
    ogg: Arc<[u8]>,
    playing: Arc<bool>,
}

impl Sound {
    pub fn play(&mut self) {}
    pub fn pause(&mut self) {}
}