use vectors::*;

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    position: Vec3<f32>,
    texcoord: Vec2<f32>,
}

impl Vertex {
    pub fn new(pos: Vec3<f32>, tex: Vec2<f32>) -> Vertex {
        Vertex {
            position: pos,
            texcoord: tex,
        }
    }
}
