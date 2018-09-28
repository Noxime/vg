use super::{Mesh, Shader, Texture};

pub struct DrawCall {
    pub mesh: Option<Mesh>,
}

impl DrawCall {
    pub fn empty() -> DrawCall { DrawCall {
        mesh: None
    } }

    pub fn set_mesh(&mut self, mesh: &Mesh) {
        self.mesh = Some(mesh.clone());
    }
}
