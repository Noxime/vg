use cpal;
use kea::audio;

use std::io::Cursor;
use std::sync::{Arc, Mutex};

use cpal::{StreamData, UnknownTypeOutputBuffer};
use lewton::inside_ogg::OggStreamReader;

pub struct Audio {
    map: Map,
}

impl Audio {
    pub fn new() -> Self {
        let events = cpal::EventLoop::new();

        // TODO: Audio switching
        let device = cpal::default_output_device().expect("No audio device");

        let mut supported_formats_range = device
            .supported_output_formats()
            .expect("error while querying formats");
        let format = supported_formats_range
            .next()
            .expect("no supported format?!")
            .with_max_sample_rate();

        let stream_id = events.build_output_stream(&device, &format).unwrap();
        events.play_stream(stream_id);

        let map: Map = Arc::new(Mutex::new(vec![]));
        let my_map = map.clone();

        std::thread::spawn(move || {
            events.run(move |_id, data| match data {
                StreamData::Output { mut buffer } => {
                    for data in &*map.lock().unwrap() {
                        data.lock().unwrap().render(&mut buffer, format.sample_rate.0);
                    }
                }
                _ => (),
            })
        });

        Audio { map: my_map }
    }
}

impl audio::Audio for Audio {
    type Clip = Clip;
    fn from_vorbis(&self, bytes: &[u8]) -> Clip {
        let bytes = Arc::new(bytes.to_vec());
        Clip::new(bytes, self.map.clone())
    }
}

type Map = Arc<Mutex<Vec<Arc<Mutex<Data>>>>>;

pub struct Clip {
    map: Map,
    data: Arc<Mutex<Data>>,
}

struct Data {
    playing: bool,
    time: f32,
    dropped: bool,
    bytes: Arc<Vec<u8>>,
    ogg: OggStreamReader<Cursor<Vec<u8>>>,
    buffer_rate: u32,
    buffer: Vec<i16>,
}

impl Data {
    fn render(&mut self, buffer: &mut UnknownTypeOutputBuffer, rate: u32) {
        if !self.playing {
            return
        }

        let chan_count = self.ogg.ident_hdr.audio_channels;
        let mut channel = 0;

        match buffer {
            UnknownTypeOutputBuffer::I16(ref mut buffer) => {
                for e in buffer.iter_mut() {
                    if channel % chan_count == 0 {
                        self.time += 1.0 / rate as f32;
                        channel = 0;
                    }
                    
                    // our time as sample index
                    let t = self.time as f64 * self.buffer_rate as f64 * chan_count as f64;
                    let sli = t.floor() as usize * chan_count as usize + channel as usize;
                    let shi = t.ceil() as usize * chan_count as usize + channel as usize;

                    // make sure we always have some (128) samples just in case
                    while shi >= self.buffer.len() {
                        if let Some(mut read) = self.ogg.read_dec_packet_itl().expect("Vorbis decoder error") {
                            self.buffer.append(&mut read);
                        }
                    }

                    let sl = self.buffer[sli];
                    let sh = self.buffer[shi];
                    let i = t.fract();
                    *e = (sl as f64 * i + sh as f64 * (1.0 - i)) as i16; 
                    channel += 1;
                }
            }
            _ => (),
        }
    }
}

impl Drop for Clip {
    fn drop(&mut self) {
        self.data.lock().unwrap().dropped = true;
    }
}

impl Clip {
    fn new(bytes: Arc<Vec<u8>>, map: Map) -> Clip {
        let ogg =
            OggStreamReader::new(Cursor::new((*bytes).clone())).expect("couldnt not make reader");

        let rate = ogg.ident_hdr.audio_sample_rate;

        let data = Arc::new(Mutex::new(Data {
            playing: false,
            time: 0.0,
            dropped: false,
            bytes,
            ogg,
            buffer_rate: rate,
            buffer: vec![],
        }));

        map.lock().unwrap().push(data.clone());

        Clip { map, data }
    }
}

impl Clone for Clip {
    fn clone(&self) -> Clip {
        Clip::new(self.data.lock().unwrap().bytes.clone(), self.map.clone())
    }
}

impl audio::Clip for Clip {
    fn play(&mut self) {
        self.data.lock().unwrap().playing = true;
    }
    fn pause(&mut self) {
        self.data.lock().unwrap().playing = false;
    }
    fn repeat(&mut self, repeat: bool) {
        unimplemented!()
    }
    fn seek(&mut self, secs: f32) {
        self.data.lock().unwrap().time = secs;
    }
    fn length(&self) -> f32 {
        unimplemented!()
    }
    fn time(&self) -> f32 {
        self.data.lock().unwrap().time
    }
    fn done(&self) -> bool {
        unimplemented!()
    }
}
