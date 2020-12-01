use super::{Env, Gfx, Light, Material, Mesh};

pub struct Graph<'a> {
    pub(crate) meshes: Vec<(&'a Mesh, &'a Material<'a>)>,
    pub(crate) lights: Vec<&'a Light>,
    pub(crate) env: Option<&'a Env>,
}

impl<'a> Graph<'a> {
    pub fn new() -> Graph<'a> {
        Graph {
            meshes: vec![],
            lights: vec![],
            env: None,
        }
    }

    pub fn mesh(&mut self, mesh: &'a Mesh, material: &'a Material<'a>) {
        self.meshes.push((mesh, material));
    }

    pub fn light(&mut self, light: &'a Light) {
        self.lights.push(light);
    }

    pub fn set_env(&mut self, env: &'a Env) {
        self.env = Some(env);
    }
}
