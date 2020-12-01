use super::{Camera, Light};

use ultraviolet::{Mat4, Vec3, Vec4};

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Uniforms {
    // pub view_position: Vec3,
    // _pad: u32,
    pub view: Mat4,
    pub projection: Mat4,
    pub resolution: Vec4,
    pub lights: [Light; 8],
    pub light_count: u32,
}

unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}

impl Uniforms {
    pub fn new() -> Uniforms {
        println!("Uniforms are {} bytes", std::mem::size_of::<Uniforms>());

        Uniforms {
            // view_position: Vec3::zero(),
            // _pad: 0,
            view: Mat4::identity(),
            projection: Mat4::identity(),
            resolution: Vec4::new(1.0, 1.0, 0.0, 0.0),
            lights: [Light::new(Vec3::zero(), Vec3::zero()); 8],
            light_count: 0,
        }
    }

    pub(crate) fn update(&mut self, size: (u32, u32), camera: Camera) {
        self.view = camera.view;

        let projection = ultraviolet::projection::perspective_wgpu_dx(
            camera.fov.to_radians(),
            size.0 as f32 / size.1 as f32,
            0.1,
            10.0,
        );
        self.projection = projection;
        self.resolution = Vec4::new(size.0 as f32, size.1 as f32, 0.0, 0.0);
    }
}
