use cpal;
use cpal::traits::{HostTrait, DeviceTrait, EventLoopTrait};
use kea::audio;
use parking_lot::{Mutex, RwLock};
use lewton::inside_ogg::OggStreamReader;

use std::collections::VecDeque;
use std::io::Cursor;
use std::sync::{Arc, Weak, atomic::AtomicBool, atomic::Ordering::Relaxed};


struct Inner {
    vorbis: Mutex<OggStreamReader<Cursor<Vec<u8>>>>,
    buffer: Mutex<VecDeque<i16>>,
    playing: AtomicBool,
    repeating: AtomicBool,
}

pub struct Sound {
    inner: Arc<Inner>,
}

impl audio::Sound for Sound {
    fn playing(&self) -> bool {
        self.inner.playing.load(Relaxed)
    }

    fn play(&mut self) {
        self.inner.playing.store(true, Relaxed);
    }

    fn pause(&mut self) {
        self.inner.playing.store(false, Relaxed);
    }

    fn repeating(&self) -> bool {
        self.inner.repeating.load(Relaxed)
    }

    fn set_repeating(&mut self, repeating: bool) {
        self.inner.repeating.store(repeating, Relaxed);
    }
}

#[derive(Copy, Clone)]
struct Settings {
    volume: f32,
    pan: f32,
}

pub struct Audio {
    sounds: Arc<Mutex<Vec<Weak<Inner>>>>,
    // thread: std::thread::JoinHandle<()>,
    settings: Arc<Mutex<Settings>>,
}

impl audio::Audio for Audio {
    type Sound = Sound;

    fn ogg(&mut self, bytes: Vec<u8>) -> Self::Sound {
        let inner = Arc::new(Inner {
            vorbis: Mutex::new(OggStreamReader::new(Cursor::new(bytes)).expect("Invalid OGG")),
            buffer: Mutex::new(VecDeque::new()),
            playing: AtomicBool::from(false),
            repeating: AtomicBool::from(false),
        });

        self.sounds.lock().push(Arc::downgrade(&inner));

        Sound {
            inner
        }
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

    fn volume(&self) -> f32 { self.settings.lock().volume }
    fn set_volume(&mut self, volume: f32) { self.settings.lock().volume = volume }

    fn pan(&self) -> f32 { self.settings.lock().pan }
    fn set_pan(&self, pan: f32) { self.settings.lock().pan = pan }
}

impl Audio {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let events = host.event_loop();

        let device = host.default_output_device().expect("No output devices");
        let mut format = device.default_output_format().expect("Format error");
        format.sample_rate.0 = 44100;

        let id = match events.build_output_stream(&device, &format) {
            Ok(id) => id,
            Err(e) => {
                println!("CPAL: your output device doesnt support 44.1khz audio, so falling back to compat mode. Expect worse sound quality");
                events.build_output_stream(&device, &device.default_output_format().expect("Format error 2")).expect("No formats")
            }
        };

        let rate = format.sample_rate.0 as usize;
        println!("CPAL: {} hz", rate);

        events.play_stream(id).expect("Failed to play stream");

        let sounds: Arc<Mutex<Vec<Weak<Inner>>>> = Arc::new(Mutex::new(vec![]));
        let thread_sounds = sounds.clone();

        let settings = Arc::new(Mutex::new(Settings {
            volume: 1.0,
            pan: 0.0,
        }));

        let _t_settings = Arc::clone(&settings);

        
        let thread = std::thread::spawn(move || {
            events.run(move |id, result| {

                let mut stream_data = match result {
                    Ok(data) => data,
                    Err(err) => {
                        eprintln!("an error occurred on stream {:?}: {}", id, err);
                        return;
                    }
                    _ => return,
                };

                match stream_data {
                    cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::U16(ref mut buffer) } => {
                        for elem in buffer.iter_mut() {
                            *elem = u16::max_value() / 2;
                        }
                    },
                    cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::I16(ref mut buffer) } => {
                        for elem in buffer.iter_mut() {
                            *elem = 0;
                        }
                    },
                    cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::F32(ref mut buffer) } => {
                        for elem in buffer.iter_mut() {
                            *elem = 0.0;
                        }
                    },
                    _ => (),
                }

                let sounds = thread_sounds.lock();
                for sound in sounds.iter() {
                    let sound = match sound.upgrade() {
                        Some(sound) => sound,
                        _ => continue
                    };

                    if !sound.playing.load(Relaxed){
                        continue
                    }

                    let load = || {
                        let mut buf = sound.buffer.lock();
                        match buf.pop_front() {
                            Some(sample) => sample,
                            None => {
                                let samples = sound.vorbis.lock().read_dec_packet_itl().expect("Decode error");
                                if let Some(samples) = samples {
                                    let mut resampled = VecDeque::new();
                                    let ogg_rate = sound.vorbis.lock().ident_hdr.audio_sample_rate as usize;
                                    let sample_count = samples.len() * rate / ogg_rate;

                                    // TODO: A better resampling method? This sounds pretty horrible, some high squaky 
                                    // sounds. Perhaps spline? or better yet, a Sinc function

                                    for i in 0..sample_count {
                                        let i = i as f64 * ogg_rate as f64 / rate as f64;
                                        let i = i.min(samples.len() as f64 - 1.0);
                                        resampled.push_back((samples[i.floor() as usize] + samples[i.ceil() as usize]) / 2);
                                    }

                                    buf.append(&mut resampled);
                                    buf.pop_front().unwrap_or(0)
                                } else {
                                    if sound.repeating.load(Relaxed) {
                                        // TODO: Repeat the track
                                    } else {
                                        sound.playing.store(false, Relaxed);
                                    }
                                    0
                                }
                            }
                        }
                    };

                    let mut i = 0;
                    match stream_data {
                        cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::U16(ref mut buffer) } => {
                            for elem in buffer.iter_mut() {
                                *elem += (load() as i32 + u16::max_value() as i32 / 2) as u16;
                                i += 1;
                            }
                        },
                        cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::I16(ref mut buffer) } => {
                            for elem in buffer.iter_mut() {
                                *elem += load();
                                i += 1;
                            }
                        },
                        cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::F32(ref mut buffer) } => {
                            for elem in buffer.iter_mut() {
                                *elem = load() as f32 / i16::max_value() as f32;
                                i += 1;
                            }
                        },
                        _ => (),
                    }
                }
            })
        });

        Audio {
            sounds,
            // thread, 
            settings,
        }
    }
}
