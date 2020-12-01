use ultraviolet::Vec3;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Light {
    position: Vec3,
    _pad0: u32,
    color: Vec3,
    _pad1: u32,
}

impl Light {
    pub fn new(position: Vec3, color: Vec3) -> Light {
        Light {
            position,
            color,
            _pad0: 0,
            _pad1: 0,
        }
    }
}

unsafe impl bytemuck::Zeroable for Light {}
unsafe impl bytemuck::Pod for Light {}
