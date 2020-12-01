use ultraviolet::{Mat4, Vec3};

pub struct Camera {
    pub(crate) view: Mat4,
    pub(crate) fov: f32,
}

impl Camera {
    pub fn new(pos: Vec3, look: Vec3, fov: f32) -> Camera {
        let view = Mat4::look_at(pos, look, Vec3::unit_y());
        Camera { view, fov }
    }
}
