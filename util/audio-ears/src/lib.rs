use kea::audio;
use cpal;

use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

pub struct Audio {
    tx: Sender<u8>,
}

impl Audio {
    pub fn new() -> Self {

        let (tx, rx) = channel();
        let events = cpal::EventLoop::new();

        // TODO: Audio switching
        let device = cpal::default_output_device().expect("No audio device");

        let mut supported_formats_range = device.supported_output_formats()
            .expect("error while querying formats");
        let format = supported_formats_range.next()
            .expect("no supported format?!")
            .with_max_sample_rate();

        let stream_id = events.build_output_stream(&device, &format).unwrap();
        events.play_stream(stream_id);

        println!("Playing, format {:#?}", format);
        
        std::thread::spawn(move || {
            use cpal::{StreamData, UnknownTypeOutputBuffer};
            let mut i = 0usize;
            events.run(move |id, data| {
                match data {
                    StreamData::Output { buffer: UnknownTypeOutputBuffer::U16(mut buffer) } => {
                        for elem in buffer.iter_mut() {
                            let t = i as f64 / format.sample_rate.0 as f64 * 400.0;
                            *elem = u16::max_value() / 2 + (t.sin() * u16::max_value() as f64 / 2.0) as u16;
                            i += 1;
                        }
                    },
                    StreamData::Output { buffer: UnknownTypeOutputBuffer::I16(mut buffer) } => {
                        for elem in buffer.iter_mut() {
                            let t = i as f64 / format.sample_rate.0 as f64 * 400.0;
                            *elem = (t.sin() * u16::max_value() as f64 / 2.0) as i16;
                            i += 1;
                        }
                    },
                    StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(mut buffer) } => {
                        for (channel, elem) in buffer.iter_mut().enumerate() {
                            let t = [
                                i as f64 / format.sample_rate.0 as f64 * 200.0, 
                                i as f64 / format.sample_rate.0 as f64 * 400.0
                            ];

                            *elem = t[channel % t.len()].sin() as f32;
                            i += 1;
                        }
                    },
                    _ => (),
                }
            })
        });

        Audio {
            tx
        }
    }
}

impl audio::Audio for Audio {
    type Clip = Clip;
    fn from_vorbis(&self, bytes: &[u8]) -> Clip {

        let mut bytes = std::io::Cursor::new(bytes);
        let start = std::time::Instant::now();
        let mut buf = vec![];
        let mut reader = lewton::inside_ogg::OggStreamReader::new(bytes).expect("Couldn't start reading OGG");
        while let Ok(Some(mut data)) = reader.read_dec_packet_itl() {
            buf.append(&mut data);
        }

        println!("Loaded all data {:?}, {} samples", start.elapsed(), buf.len());

        Clip::new(Arc::new(buf))
    }
}

pub struct Clip {
    data: Arc<Vec<i16>>,
}

impl Clip {
    fn new(data: Arc<Vec<i16>>) -> Clip {

        Clip {
            data,
        }
    }
}

impl Clone for Clip {
    fn clone(&self) -> Clip {
        Clip::new(self.data.clone())
    }
}

impl audio::Clip for Clip {
    fn play(&mut self) {}
    fn pause(&mut self) {}
    fn repeat(&mut self, repeat: bool) {}
    fn seek(&mut self, secs: f32) {}
    fn length(&self) -> f32 { unimplemented!() }
    fn time(&self) -> f32 { unimplemented!() }
    fn done(&self) -> bool { unimplemented!() }
}