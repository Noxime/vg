use super::Gfx;

use ultraviolet::{Vec2, Vec3};
use wgpu::*;

#[derive(Debug, Clone, Copy)]
pub struct TextureSettings {
    /// F32, U8
    pub hdr: bool,
    /// sRGB or Linear color
    pub linear: bool,
    pub channels: u8, // 1 or 4
}

impl Default for TextureSettings {
    fn default() -> Self {
        TextureSettings {
            hdr: false,
            linear: false,
            channels: 4,
        }
    }
}

pub struct Texture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: TextureView,
    pub(crate) sampler: Sampler,
    pub(crate) size: Extent3d,
    pub(crate) bpp: u32,
}

impl Texture {
    pub fn new(gfx: &Gfx, size: (u32, u32), data: &[u8], s: TextureSettings) -> Texture {
        assert_eq!(
            size.0 * size.1 * if s.hdr { 4 } else { 1 } * s.channels as u32,
            data.len() as u32,
            "Texture data - size mismatch"
        );

        let size = Extent3d {
            width: size.0,
            height: size.1,
            depth: 1,
        };

        let format = match (s.channels, s.linear, s.hdr) {
            (1, true, false) => TextureFormat::R8Unorm,
            (1, true, true) => TextureFormat::R32Float,
            (4, false, false) => TextureFormat::Rgba8UnormSrgb,
            (4, true, false) => TextureFormat::Rgba8Unorm,
            (4, true, true) => TextureFormat::Rgba32Float,
            _ => panic!("Unsupported texture format ({:?}", s),
        };

        println!(
            "Uploading texture of {}x{} with {}kB ({:?})",
            size.width,
            size.height,
            data.len() / 1024,
            format,
        );

        let bpp = s.channels as u32 * if s.hdr { 4 } else { 1 };

        let mip_level_count = (size.width.max(size.height) as f32).log2().floor() as u32 + 1;

        let texture = gfx.device.create_texture(&TextureDescriptor {
            size,
            mip_level_count,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            label: Some("texture"),
        });

        let view = texture.create_view(&Default::default());
        let sampler = gfx.device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            anisotropy_clamp: Some(std::num::NonZeroU8::new(8).unwrap()),
            ..Default::default()
        });

        let t = Texture {
            texture,
            view,
            sampler,
            size,
            bpp,
        };

        t.update(gfx, data);

        t
    }

    pub(crate) fn update(&self, gfx: &Gfx, data: &[u8]) {
        let mip_level_count = (self.size.width.max(self.size.height) as f32)
            .log2()
            .floor() as u32
            + 1;
        let mut size = self.size;

        // upload main data
        gfx.queue.write_texture(
            TextureCopyView {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            &data,
            TextureDataLayout {
                offset: 0,
                bytes_per_row: self.bpp * size.width,
                rows_per_image: size.height,
            },
            size,
        );

        let mut img = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
            size.width,
            size.height,
            data.to_vec(),
        )
        .unwrap();

        for mip_level in 1..mip_level_count {
            size.width /= 2;
            size.height /= 2;

            img = image::imageops::resize(
                &img,
                size.width,
                size.height,
                image::imageops::FilterType::Nearest,
            );

            gfx.queue.write_texture(
                TextureCopyView {
                    texture: &self.texture,
                    mip_level,
                    origin: Origin3d::ZERO,
                },
                img.as_raw(),
                TextureDataLayout {
                    offset: 0,
                    bytes_per_row: self.bpp * size.width,
                    rows_per_image: size.height,
                },
                size,
            );
        }
    }

    pub(crate) fn new_depth(device: &Device, size: (u32, u32)) -> Self {
        let size = Extent3d {
            width: size.0,
            height: size.1,
            depth: 1,
        };
        let texture = device.create_texture(&TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsage::SAMPLED | TextureUsage::OUTPUT_ATTACHMENT,
            label: Some("depth texture"),
        });
        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            compare: Some(CompareFunction::LessEqual),
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            label: Some("depth texture sampler"),
            ..Default::default()
        });

        Texture {
            size,
            texture,
            view,
            sampler,
            bpp: 4,
        }
    }

    pub(crate) fn new_render(device: &Device, size: (u32, u32), format: TextureFormat) -> Self {
        let size = Extent3d {
            width: size.0,
            height: size.1,
            depth: 1,
        };
        let texture = device.create_texture(&TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsage::SAMPLED | TextureUsage::OUTPUT_ATTACHMENT,
            label: Some("render texture"),
        });
        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            compare: None,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            label: Some("render texture sampler"),
            ..Default::default()
        });

        Texture {
            size,
            texture,
            view,
            sampler,
            bpp: 4 * 4,
        }
    }
}

/// Environment map, cubemap
pub struct Env {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: TextureView,
    pub(crate) sampler: Sampler,
    pub(crate) size: Extent3d,
    pub(crate) bind_group: BindGroup,
}

fn make_face(
    size: i32,
    ibl_size: i32,
    meta: &image::codecs::hdr::HdrMetadata,
    img: &Vec<image::Rgb<f32>>,
    swizzle: impl Fn(f32, f32) -> Vec3,
) -> (Vec<f32>, Vec<f32>) {
    let hsize = size / 2;
    let dims = Vec2::new(meta.width as f32, meta.height as f32);

    let mut res: Vec<f32> = vec![];
    for y in -hsize..hsize {
        for x in -hsize..hsize {
            let v: Vec3 = swizzle(x as f32 / hsize as f32, y as f32 / hsize as f32).normalized();
            let uv = Vec2::new(v.z.atan2(v.x), v.y.asin()) * Vec2::new(0.1591, 0.3183)
                + Vec2::broadcast(0.5);
            assert!(uv.x > 0.0 && uv.x < 1.0);
            assert!(uv.y > 0.0 && uv.y < 1.0);
            let uv = uv * dims;
            let i = uv.x as u32 + uv.y as u32 * meta.width;
            // let i = uv.y as u32 + uv.x as u32 * meta.height;
            let p = img[i as usize];
            res.push(p[0]);
            res.push(p[1]);
            res.push(p[2]);
            res.push(1.0);
        }
    }

    let hsize = ibl_size / 2;

    let mut ibl: Vec<f32> = vec![];
    for y in -hsize..hsize {
        for x in -hsize..hsize {
            let v: Vec3 = swizzle(x as f32 / hsize as f32, y as f32 / hsize as f32).normalized();

            let mut irradiance = Vec3::zero();
            let right = Vec3::unit_y().cross(v);
            let up = v.cross(right);

            // let sample_delta = 0.025;
            let mut nr_samples = 0.0;

            for phi in 0..2048 {
                let phi = phi as f32 / 2048.0 * 3.141592 * 2.0;
                for theta in 0..512 {
                    let theta = theta as f32 / 512.0 * 3.141592 * 0.5;
                    let tangent_sample = Vec3::new(
                        theta.sin() * phi.cos(),
                        theta.sin() * phi.sin(),
                        theta.cos(),
                    );
                    let sample_vec =
                        tangent_sample.x * right + tangent_sample.y * up + tangent_sample.z * v;

                    let uv = Vec2::new(sample_vec.z.atan2(sample_vec.x), sample_vec.y.asin())
                        * Vec2::new(0.1591, 0.3183)
                        + Vec2::broadcast(0.5);
                    assert!(uv.x > 0.0 && uv.x < 1.0);
                    assert!(uv.y > 0.0 && uv.y < 1.0);
                    let uv = uv * dims;
                    let i = uv.x as u32 + uv.y as u32 * meta.width;
                    // let i = uv.y as u32 + uv.x as u32 * meta.height;
                    let p = img[i as usize];
                    irradiance +=
                        Vec3::new(p[0], p[1], p[2]) * theta.cos() * theta.sin() * 3.141592;
                    nr_samples += 1.0;
                }
            }

            ibl.push(irradiance[0] / nr_samples);
            ibl.push(irradiance[1] / nr_samples);
            ibl.push(irradiance[2] / nr_samples);
            ibl.push(1.0);
        }
    }

    (res, ibl)
}

impl Env {
    pub fn new_hdri(gfx: &mut Gfx, img: &str) -> Self {
        let img = std::io::BufReader::new(std::fs::File::open(img).unwrap());
        let de = image::codecs::hdr::HdrDecoder::new(img).unwrap();
        let meta = de.metadata();
        let img = de.read_image_hdr().unwrap();

        let size = 512i32;
        let ibl_size = 8;

        let f = make_face(size, ibl_size, &meta, &img, |x, y| Vec3::new(x, y, 1.0));
        let b = make_face(size, ibl_size, &meta, &img, |x, y| Vec3::new(-x, y, -1.0));
        let r = make_face(size, ibl_size, &meta, &img, |x, y| Vec3::new(1.0, y, -x));
        let l = make_face(size, ibl_size, &meta, &img, |x, y| Vec3::new(-1.0, y, x));
        let u = make_face(size, ibl_size, &meta, &img, |x, y| Vec3::new(-y, -1.0, x));
        let d = make_face(size, ibl_size, &meta, &img, |x, y| Vec3::new(y, 1.0, x));

        Self::new(
            gfx,
            (size as u32, size as u32),
            [
                [
                    bytemuck::cast_slice(f.0.as_slice()),
                    bytemuck::cast_slice(f.1.as_slice()),
                ],
                [
                    bytemuck::cast_slice(b.0.as_slice()),
                    bytemuck::cast_slice(b.1.as_slice()),
                ],
                [
                    bytemuck::cast_slice(u.0.as_slice()),
                    bytemuck::cast_slice(u.1.as_slice()),
                ],
                [
                    bytemuck::cast_slice(d.0.as_slice()),
                    bytemuck::cast_slice(d.1.as_slice()),
                ],
                [
                    bytemuck::cast_slice(l.0.as_slice()),
                    bytemuck::cast_slice(l.1.as_slice()),
                ],
                [
                    bytemuck::cast_slice(r.0.as_slice()),
                    bytemuck::cast_slice(r.1.as_slice()),
                ],
            ],
        )
    }

    /// positive XYZ, negative XYZ. Expects Rgba<f32>
    pub fn new(gfx: &mut Gfx, size: (u32, u32), data: [[&[u8]; 2]; 6]) -> Self {
        println!(
            "Uploading env map with {}kB",
            data.iter().map(|s| s.len()).sum::<usize>() / 1024
        );
        let size = Extent3d {
            width: size.0,
            height: size.1,
            depth: 6,
        };
        let texture = gfx.device.create_texture(&TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            label: Some("env texture"),
        });
        let ibl_size = Extent3d {
            width: 8,
            height: 8,
            depth: 6,
        };
        let ibl_texture = gfx.device.create_texture(&TextureDescriptor {
            size: ibl_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            label: Some("ibl texture"),
        });

        for (i, image) in data.iter().enumerate() {
            gfx.queue.write_texture(
                TextureCopyView {
                    texture: &texture,
                    mip_level: 0,
                    origin: Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                },
                &image[0][..(16 * size.width * size.height) as usize],
                TextureDataLayout {
                    offset: 0,
                    bytes_per_row: 16 * size.width,
                    rows_per_image: size.height,
                },
                Extent3d { depth: 1, ..size },
            );

            gfx.queue.write_texture(
                TextureCopyView {
                    texture: &ibl_texture,
                    mip_level: 0,
                    origin: Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                },
                &image[1][..(16 * ibl_size.width * ibl_size.height) as usize],
                TextureDataLayout {
                    offset: 0,
                    bytes_per_row: 16 * ibl_size.width,
                    rows_per_image: ibl_size.height,
                },
                Extent3d {
                    depth: 1,
                    ..ibl_size
                },
            );
        }

        let view = texture.create_view(&TextureViewDescriptor {
            label: Some("env sampler"),
            dimension: Some(TextureViewDimension::Cube),
            ..Default::default()
        });

        let ibl_view = ibl_texture.create_view(&TextureViewDescriptor {
            label: Some("ibl sampler"),
            dimension: Some(TextureViewDimension::Cube),
            ..Default::default()
        });

        let sampler = gfx.device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            compare: None,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            label: Some("env sampler"),
            ..Default::default()
        });

        let bind_group = gfx.device.create_bind_group(&BindGroupDescriptor {
            label: Some("env bind group"),
            layout: &gfx.env_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&ibl_view),
                },
            ],
        });

        Env {
            size,
            texture,
            view,
            sampler,
            bind_group,
        }
    }
}
