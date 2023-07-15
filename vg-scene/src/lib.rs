use glam::{Mat4, Vec4};

pub type Color = Vec4;

/// A game object that can have all sorts of properties on it
pub struct Object {
    pub transform: Space<Mat4>,
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
}

/// Coordinate space variant
pub enum Space<T = ()> {
    /// Data is in world space
    World(T),
    /// Data is in view space
    View(T),
}

pub struct Stroke {
    /// Thickness of the stroke
    pub thickness: Space<f32>,
    /// Colors of the stroke, interpolated evenly around the entire stroke perimeter
    pub colors: Vec<Color>,
}

pub struct Fill {
    pub color: Color,
}
