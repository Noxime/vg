use components::Component;
use graphics::*;
use vectors::*;

pub struct SpriteRenderer {
    mesh: Mesh,
}

impl SpriteRenderer {
    pub fn new() -> SpriteRenderer {
        SpriteRenderer {
            mesh: Mesh::new(vec![
                Vertex::new(Vec3::new(0.0, 0.0, 0.0), Vec2::new(0.0, 0.0)),
                Vertex::new(Vec3::new(1.0, 0.0, 0.0), Vec2::new(1.0, 0.0)),
                Vertex::new(Vec3::new(0.0, 1.0, 0.0), Vec2::new(0.0, 1.0)),
                Vertex::new(Vec3::new(1.0, 1.0, 0.0), Vec2::new(0.0, 0.0)),
                Vertex::new(Vec3::new(0.0, 1.0, 0.0), Vec2::new(0.0, 0.0)),
                Vertex::new(Vec3::new(1.0, 0.0, 0.0), Vec2::new(0.0, 0.0)),
            ]),
        }
    }
}

impl Component for SpriteRenderer {
    fn render(&mut self) -> Option<DrawCall> {
        let mut call = DrawCall::empty();
        call.set_mesh(&self.mesh);
        Some(call)
    }
}
