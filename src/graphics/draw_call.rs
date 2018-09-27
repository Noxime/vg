use super::{Mesh, Shader, Texture};

pub struct DrawCall {

}

impl DrawCall {
    pub fn empty() -> DrawCall {
        DrawCall {}
    }

    pub fn with_mesh(mut self, mesh: Mesh) -> Self {
        self
    }
}