use crate::{gfx::Source as TextureSource, *};

use async_trait::async_trait;

pub struct Asset {
    pub(crate) bytes: Vec<u8>,
}

impl Asset {
    pub async fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
}

pub struct PngSource(Asset);

impl PngSource {
    pub fn new(asset: Asset) -> PngSource {
        PngSource(asset)
    }
}

#[async_trait]
impl TextureSource for PngSource {
    async fn load(&self) -> (Vec<Color>, Size) {
        let bytes = self.0.bytes().await;
        let decoder = png::Decoder::new(&bytes[..]);
        let (info, mut reader) = decoder.read_info().expect("Invalid PNG");
        assert_eq!(
            info.bit_depth,
            png::BitDepth::Eight,
            "Only 8-bit PNGs supported at this time"
        );

        let size = Size::new(info.width, info.height);
        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf).unwrap();

        (
            match info.color_type {
                png::ColorType::RGB => {
                    debug!("Loaded RGB PNG");
                    let mut vec = Vec::with_capacity(buf.len() * 4 / 3);
                    for g in buf.chunks(3) {
                        vec.push(Color::new(
                            g[0] as f32 / 255.0,
                            g[1] as f32 / 255.0,
                            g[2] as f32 / 255.0,
                            1.0,
                        ));
                    }
                    vec
                }
                png::ColorType::RGBA => {
                    debug!("Loaded RGBA PNG");
                    let mut vec = Vec::with_capacity(buf.len());
                    for g in buf.chunks(4) {
                        vec.push(Color::new(
                            g[0] as f32 / 255.0,
                            g[1] as f32 / 255.0,
                            g[2] as f32 / 255.0,
                            g[3] as f32 / 255.0,
                        ));
                    }
                    vec
                }
                t => todo!("PNG with color type {:?} are not supported", t),
            },
            size,
        )
    }
}

pub fn png(asset: Asset) -> PngSource {
    PngSource::new(asset)
}
