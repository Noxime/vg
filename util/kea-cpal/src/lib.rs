use cpal;
use cpal::traits::{HostTrait, DeviceTrait, EventLoopTrait};
use kea::audio;

use std::sync::Arc;

pub struct Audio {
    host: cpal::Host,
    device: cpal::Device,
    thread: std::thread::JoinHandle<()>,
}

impl audio::Audio for Audio {
    fn ogg(&mut self, bytes: Vec<u8>) -> audio::Sound {
        audio::Sound {
            playing: false,
        }
    }

    fn output(&self) -> Option<audio::Output> {
        let (name, format) = match (self.device.name(), self.device.default_output_format()) {
            (Ok(n), Ok(f)) => (n, f),
            e => {
                println!("CPAL ERROR: {:?}", e);
                return None;
            }
        };

        Some(audio::Output {
            name,
            channels: format.channels as usize,
            samples: format.sample_rate.0 as usize,
            bits: format.data_type.sample_size() * 8,
        })
    }

    fn outputs(&self) -> Vec<audio::Output> {
        use std::iter::FromIterator;

        let devices = match self.host.output_devices() {
            Ok(v) => v,
            Err(e) => {
                println!("CPAL ERROR: {}", e);
                return vec![]
            }
        };

        let mut outs = vec![];

        for device in devices {
            let (name, formats) = match (device.name(), device.supported_output_formats()) {
                (Ok(n), Ok(f)) => (n, f),
                e => {
                    println!("CPAL ERROR: <unk>");
                    continue;
                }
            };

            for format in formats {
                let format = format.with_max_sample_rate();
                outs.push(audio::Output {
                    name: name.clone(),
                    channels: format.channels as usize,
                    samples: format.sample_rate.0 as usize,
                    bits: format.data_type.sample_size() * 8,
                });
            }
        }

        outs
    }

    fn set_output(&mut self, output: impl AsRef<audio::Output>) -> Result<(), ()> {
        let output = output.as_ref();

        let device = self.host.output_devices().map_err(|_| ())?.find(|v| {
            if let Ok(name) = v.name() {
                name == output.name
            } else { false }
        });

        let device = match device {
            Some(d) => d,
            None => return Err(())
        };

        let format = device.supported_output_formats().map_err(|_| ())?.find(|v| {
            v.channels as usize == output.channels &&
            v.data_type.sample_size() * 8 == output.bits &&
            (v.max_sample_rate.0 as usize >= output.samples && v.min_sample_rate.0 as usize <= output.samples)
        });

        let format = match format {
            Some(f) => f.with_max_sample_rate(),
            None => return Err(())
        };

        self.device = device;

        Ok(())
    }

    fn volume(&self) -> f32 { unimplemented!() }
    fn set_volume(&mut self, volume: f32) { unimplemented!() }

    fn pan(&self) -> f32 { unimplemented!() }
    fn set_pan(&self, pan: f32) { unimplemented!() }
}

impl Audio {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let events = host.event_loop();

        let device = host.default_output_device().expect("No output devices");
        let format = device.default_output_format().expect("Format error");

        let id = events.build_output_stream(&device, &format).unwrap();
        events.play_stream(id).expect("Failed to play stream");

        let thread = std::thread::spawn(move || {
            events.run(move |id, result| {
                let stream_data = match result {
                    Ok(data) => data,
                    Err(err) => {
                        eprintln!("an error occurred on stream {:?}: {}", id, err);
                        return;
                    }
                    _ => return,
                };

                match stream_data {
                    cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer) } => {
                        for elem in buffer.iter_mut() {
                            *elem = u16::max_value() / 2;
                        }
                    },
                    cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer) } => {
                        for elem in buffer.iter_mut() {
                            *elem = 0;
                        }
                    },
                    cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer) } => {
                        for elem in buffer.iter_mut() {
                            *elem = 0.0;
                        }
                    },
                    _ => (),
                }
            })
        });

        Audio {
            host,
            device,
            thread,
        }
    }
}
