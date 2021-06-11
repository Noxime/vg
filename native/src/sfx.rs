use std::{cell::Cell, io::Cursor, rc::Rc, time::Duration};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, Stream, StreamConfig,
};
use futures::compat::Future01CompatExt;
use lewton::{
    header::{read_header_comment, read_header_ident, read_header_setup},
    inside_ogg::async_api::{HeadersReader, OggStreamReader},
};
use oddio::{Frames, FramesSignal, Handle, Sample, Signal, SpatialScene, StreamControl};
use tracing::{debug, error, trace};

use crate::assets::Cache;

pub struct Sfx {
    scene: Handle<SpatialScene>,
    stream: Stream,
}

impl Sfx {
    pub fn new() -> Sfx {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output devices");
        let sample_rate = device.default_output_config().unwrap().sample_rate();

        debug!(
            "Using: {}, sample rate: {}",
            device.name().unwrap_or_default(),
            sample_rate.0
        );

        let config = StreamConfig {
            channels: 2,
            sample_rate,
            buffer_size: BufferSize::Default,
        };

        let (scene_handle, scene) = oddio::split(oddio::SpatialScene::new(sample_rate.0, 0.1));

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _| {
                    let frames = oddio::frame_stereo(data);
                    oddio::run(&scene, sample_rate.0, frames);
                },
                |err| {
                    error!("Audio output error: {}", err);
                },
            )
            .unwrap();

        stream.play().unwrap();

        Sfx {
            scene: scene_handle,
            stream,
        }
    }

    pub async fn play_sound(&mut self, asset: Rc<Cache>) {
        use futures::{compat::Stream01CompatExt, StreamExt};
        let mut packet_reader =
            ogg::reading::async_api::PacketReader::new(asset.start_read().await).compat();

        let ident = read_header_ident(&packet_reader.next().await.unwrap().unwrap().data).unwrap();
        let comment =
            read_header_comment(&packet_reader.next().await.unwrap().unwrap().data).unwrap();
        let setup = read_header_setup(
            &packet_reader.next().await.unwrap().unwrap().data,
            ident.audio_channels,
            (ident.blocksize_0, ident.blocksize_1),
        )
        .unwrap();

        let sample_rate = ident.audio_sample_rate;

        let mut decoder =
            OggStreamReader::from_pck_rdr(packet_reader.into_inner(), (ident, comment, setup))
                .compat();

        let mut buf = vec![];
        let signal = oddio::Stream::<f32>::new(sample_rate, 1024);
        debug!("Playing {} at {}hz", asset.path.display(), sample_rate);

        let mut handle =
            self.scene
                .control()
                .play(signal, [0.0, 0.0, 1.0].into(), [0.0; 3].into(), 1000.0);

        tokio::spawn(async move {
            loop {
                // our decoded buffer is empty, so we better decode some more
                if buf.is_empty() {
                    match decoder.next().await {
                        Some(Ok(frames)) => {
                            for s in frames[0].iter() {
                                buf.push(*s as f32 / std::i16::MAX as f32);
                            }
                        }
                        Some(Err(e)) => {
                            error!("Error during vorbis decode: {:?}", e);
                            break;
                        }
                        None => break,
                    }
                } else {
                    // we have enough decoded samples in buffer, just play them
                    let n = {
                        let mut control = handle.control::<oddio::Stream<_>, _>();
                        let n = control.write(&buf);
                        buf.drain(..n);
                        n
                    };
                    // sleep for half-n time
                    tokio::time::sleep(Duration::from_micros(
                        500_000 * n as u64 / sample_rate as u64,
                    ))
                    .await;
                }
            }
        });
    }
}
