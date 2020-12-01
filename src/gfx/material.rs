use super::{Gfx, Texture};

use ultraviolet::Vec4;

pub enum Source<'a, T> {
    Value(T),
    Texture(&'a Texture),
}

impl<'a, T: Default> Default for Source<'a, T> {
    fn default() -> Self {
        Self::Value(Default::default())
    }
}

impl<'a, T> From<T> for Source<'a, T> {
    fn from(t: T) -> Self {
        Self::Value(t)
    }
}

pub(crate) trait AsBytes {
    fn bytes(&self) -> [u8; 4];
}

impl AsBytes for f32 {
    fn bytes(&self) -> [u8; 4] {
        let x = (self * 255.0) as u8;
        [x; 4]
    }
}

impl AsBytes for Vec4 {
    fn bytes(&self) -> [u8; 4] {
        [
            (self[0] * 255.0) as u8,
            (self[1] * 255.0) as u8,
            (self[2] * 255.0) as u8,
            (self[3] * 255.0) as u8,
        ]
    }
}

pub struct Material<'a> {
    pub color: Source<'a, Vec4>,
    pub reflectance: Source<'a, f32>,
    pub roughness: Source<'a, f32>,
    pub metallic: Source<'a, f32>,
    pub clear_coat: Source<'a, f32>,
    pub clear_coat_roughness: Source<'a, f32>,
    pub normal: Source<'a, ()>,
    pub emission: Source<'a, Vec4>,
}

impl<'a> Default for Material<'a> {
    fn default() -> Self {
        // These are mostly based on google Filament's defaults
        Self {
            color: Vec4::new(0.8, 0.8, 0.8, 1.0).into(),
            reflectance: 0.5.into(),
            roughness: 0.2.into(),
            metallic: 0.0.into(),
            clear_coat: 0.0.into(),
            clear_coat_roughness: 0.0.into(),
            normal: ().into(),
            emission: Vec4::zero().into(),
        }
    }
}
