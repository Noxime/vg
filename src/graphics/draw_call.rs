use super::{Mesh, Shader, Texture};

pub struct DrawCall {}

impl DrawCall {
    pub fn empty() -> DrawCall { DrawCall {} }

    pub fn set_mesh(&mut self, mesh: &Mesh) { 
        // TODO
    }
}
