use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    time::Duration,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, Stream, StreamConfig,
};
use dashmap::DashMap;
use lewton::{
    header::{read_header_comment, read_header_ident, read_header_setup},
    inside_ogg::async_api::OggStreamReader,
};
use oddio::{Handle, Mixer, Spatial, SpatialScene, Stop};
use tracing::{debug, error, warn};

use crate::assets::Cache;

pub struct Sfx {
    scene: Handle<Mixer<[f32; 2]>>,
    stream: Stream,
    dead_sound_tx: Sender<Handle<Stop<oddio::Stream<[f32; 2]>>>>,
    dead_sounds: Receiver<Handle<Stop<oddio::Stream<[f32; 2]>>>>,
    active_streams: usize,
    hack_bgm: bool,
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

        let (scene_handle, scene) = oddio::split(oddio::Mixer::new());

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

        let (dead_sound_tx, dead_sounds) = mpsc::channel();

        Sfx {
            scene: scene_handle,
            stream,
            dead_sound_tx,
            dead_sounds,
            active_streams: 0,
            hack_bgm: false,
        }
    }

    pub async fn play_sound(&mut self, asset: Arc<Cache>, looping: bool) {
        if asset.path.ends_with("bgm.ogg") {
            if self.hack_bgm {
                return;
            } else {
                self.hack_bgm = true;
            }
        }

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

        let signal = oddio::Stream::<[f32; 2]>::new(sample_rate, 1024);
        debug!("Playing {} at {}hz", asset.path.display(), sample_rate);

        let mut handle = match self.dead_sounds.try_recv() {
            Ok(handle) => handle,
            Err(err) => {
                debug!(
                    "Failed to get existing audio stream handle: {} ({} existing streams)",
                    err, self.active_streams
                );
                self.active_streams += 1;
                self.scene.control().play(signal)
            }
        };

        let (tx, mut rx) = tokio::sync::mpsc::channel(64);
        let dead_sound_tx = self.dead_sound_tx.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_micros(1)).await;

                match decoder.next().await {
                    Some(Ok(frames)) => {
                        if let Err(e) = tx.send(frames).await {
                            error!("Failed to send audio frames to stream: {}", e);
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        error!("Vorbis decode error: {:?}", e);
                        break;
                    }
                    None => {
                        if looping {
                            debug!("Restarting loop");
                            let mut packet_reader = ogg::reading::async_api::PacketReader::new(
                                asset.start_read().await,
                            )
                            .compat();

                            let ident = read_header_ident(
                                &packet_reader.next().await.unwrap().unwrap().data,
                            )
                            .unwrap();
                            let comment = read_header_comment(
                                &packet_reader.next().await.unwrap().unwrap().data,
                            )
                            .unwrap();
                            let setup = read_header_setup(
                                &packet_reader.next().await.unwrap().unwrap().data,
                                ident.audio_channels,
                                (ident.blocksize_0, ident.blocksize_1),
                            )
                            .unwrap();
                            decoder = OggStreamReader::from_pck_rdr(
                                packet_reader.into_inner(),
                                (ident, comment, setup),
                            )
                            .compat();
                        } else {
                            debug!("Done with decoding");
                            break;
                        }
                    }
                }
            }
        });

        tokio::spawn(async move {
            let mut buf = vec![];
            let mut done = false;
            loop {
                let sleep = {
                    let mut control = handle.control::<oddio::Stream<_>, _>();
                    let n = control.write(&buf);
                    buf.drain(..n);
                    Duration::from_micros(250_000 * n as u64 / sample_rate as u64)
                };

                if done && buf.is_empty() {
                    debug!("Done playing sound");
                    break;
                }

                match tokio::time::timeout(sleep, async {
                    if let Some(frames) = rx.recv().await {
                        for s in &frames[0] {
                            let f = *s as f32 / std::i16::MAX as f32;
                            buf.push([f; 2]);
                        }
                        false
                    } else {
                        true
                    }
                })
                .await
                {
                    Ok(true) => done = true,
                    Err(_) => {
                        if buf.is_empty() && !done {
                            warn!("What the fuck");
                        }
                    }
                    Ok(false) => (),
                }
            }

            let _ = dead_sound_tx.send(handle);
        });
    }
}
