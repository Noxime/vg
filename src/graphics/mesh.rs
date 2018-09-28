// This mesh uses a usize id to do equality and hashing, so even though meshes
// might have same verticies, their ID's will always be unique

// TODO: Figure out if Ordering::Relaxed is the way to go (honestly no idea)

use graphics::Vertex;
use graphics::upload_mesh;
use std::sync::atomic::*;

lazy_static! {
    static ref COUNTER: AtomicUsize = AtomicUsize::new(0);
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Mesh {
    pub id: usize,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>) -> Mesh {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        upload_mesh(id, &vertices);
        Mesh { id }
    }
}
