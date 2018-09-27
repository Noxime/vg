use components::Component;
use graphics::*;
use vectors::*;

pub struct SpriteRenderer {
    mesh: Option<Mesh>,
}

impl SpriteRenderer {
    pub fn new() -> SpriteRenderer { SpriteRenderer {
        mesh: None,
    } }
}

impl Component for SpriteRenderer {
    fn render_init(&mut self) {
        self.mesh = Some(create_mesh(vec![
            Vertex::new(Vec3::new(0.0, 0.0, 0.0), Vec2::new(0.0, 0.0)),
            Vertex::new(Vec3::new(1.0, 0.0, 0.0), Vec2::new(1.0, 0.0)),
            Vertex::new(Vec3::new(0.0, 1.0, 0.0), Vec2::new(0.0, 1.0)),
        ]));
    }

    fn render(&mut self) -> Option<DrawCall> {
        let mut call = DrawCall::empty();
        if let Some(ref m) = self.mesh {
            call.set_mesh(m)
        }
        Some(call)
    }

    fn render_destroy(&mut self) {

    }
}
