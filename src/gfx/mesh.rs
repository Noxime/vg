use super::Gfx;
use wgpu::util::*;
use wgpu::*;

use ultraviolet::{Vec2, Vec3};

use std::collections::HashMap;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    position: Vec3,
    normal: Vec3,
    uv0: Vec2,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    pub(crate) const STRIDE: BufferAddress = std::mem::size_of::<Vertex>() as BufferAddress;
    const OFFSET_POS: BufferAddress = 0;
    const OFFSET_NRM: BufferAddress =
        Self::OFFSET_POS + std::mem::size_of::<Vec3>() as BufferAddress;
    const OFFSET_UV0: BufferAddress =
        Self::OFFSET_NRM + std::mem::size_of::<Vec3>() as BufferAddress;

    pub const fn new(x: f32, y: f32, z: f32) -> Vertex {
        Vertex {
            position: Vec3::new(x, y, z),
            normal: Vec3::new(0.0, 0.0, 0.0),
            uv0: Vec2::new(0.0, 0.0),
        }
    }

    pub fn with_normal(&self, normal: Vec3) -> Self {
        Vertex { normal, ..*self }
    }

    pub fn with_uv(&self, uv: Vec2) -> Self {
        Vertex { uv0: uv, ..*self }
    }

    pub(crate) fn desc<'a>() -> VertexBufferDescriptor<'a> {
        VertexBufferDescriptor {
            stride: Vertex::STRIDE,
            step_mode: InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: Self::OFFSET_POS,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: Self::OFFSET_NRM,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: Self::OFFSET_UV0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float2,
                },
            ],
        }
    }
}

pub struct Mesh {
    pub(crate) count: u32,
    pub(crate) vertex_buffer: Buffer,
    pub(crate) index_buffer: Buffer,
}

impl Mesh {
    pub fn new(gfx: &mut Gfx, vertices: &[Vertex], indicies: &[u32]) -> Mesh {
        println!(
            "Uploading mesh with {}/{} verts/faces",
            vertices.len(),
            indicies.len() / 3
        );

        let vertex_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(vertices),
            usage: BufferUsage::VERTEX,
        });

        let index_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(indicies),
            usage: BufferUsage::INDEX,
        });

        Mesh {
            vertex_buffer,
            index_buffer,
            count: indicies.len() as u32,
        }
    }

    /// Create a new ground plane (10x10)
    pub fn plane(gfx: &mut Gfx) -> Mesh {
        Self::new(
            gfx,
            &[
                Vertex::new(5.0, -1.0, 5.0)
                    .with_normal(Vec3::unit_y())
                    .with_uv(Vec2::new(1.0, 1.0)),
                Vertex::new(-5.0, -1.0, 5.0)
                    .with_normal(Vec3::unit_y())
                    .with_uv(Vec2::new(-1.0, 1.0)),
                Vertex::new(5.0, -1.0, -5.0)
                    .with_normal(Vec3::unit_y())
                    .with_uv(Vec2::new(1.0, -1.0)),
                Vertex::new(-5.0, -1.0, -5.0)
                    .with_normal(Vec3::unit_y())
                    .with_uv(Vec2::new(-1.0, -1.0)),
            ],
            &[0, 2, 1, 3, 1, 2],
        )
    }

    /// Create a new icosphere with `level` subdivisons. Be careful with this number, the face count is 20*4^level
    pub fn sphere(gfx: &mut Gfx, level: usize) -> Mesh {
        let t = (1.0 + 5.0f32.sqrt()) / 2.0;
        let mut vertices = vec![
            Vertex::new(-1.0, t, 0.0),
            Vertex::new(1.0, t, 0.0),
            Vertex::new(-1.0, -t, 0.0),
            Vertex::new(1.0, -t, 0.0),
            Vertex::new(0.0, -1.0, t),
            Vertex::new(0.0, 1.0, t),
            Vertex::new(0.0, -1.0, -t),
            Vertex::new(0.0, 1.0, -t),
            Vertex::new(t, 0.0, -1.0),
            Vertex::new(t, 0.0, 1.0),
            Vertex::new(-t, 0.0, -1.0),
            Vertex::new(-t, 0.0, 1.0),
        ];

        let mut indicies = vec![
            0, 11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11, 1, 5, 9, 5, 11, 4, 11, 10, 2, 10, 7,
            6, 7, 1, 8, 3, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6, 8, 3, 8, 9, 4, 9, 5, 2, 4, 11, 6, 2, 10,
            8, 6, 7, 9, 8, 1u32,
        ];

        let mut cache = HashMap::new();

        // subdivide the shape
        for _ in 0..level {
            let mut indicies2 = vec![];
            for tri in indicies.chunks_exact(3) {
                let a = middle(tri[0], tri[1], &mut cache, &mut vertices);
                let b = middle(tri[1], tri[2], &mut cache, &mut vertices);
                let c = middle(tri[2], tri[0], &mut cache, &mut vertices);

                indicies2.push(tri[0]);
                indicies2.push(a);
                indicies2.push(c);
                indicies2.push(tri[1]);
                indicies2.push(b);
                indicies2.push(a);
                indicies2.push(tri[2]);
                indicies2.push(c);
                indicies2.push(b);
                indicies2.push(a);
                indicies2.push(b);
                indicies2.push(c);
            }
            indicies = indicies2;
        }

        vertices.iter_mut().for_each(|v| {
            v.position.normalize();
            v.normal = v.position;
            v.uv0 = Vec2::new(v.position.y, v.position.x.atan2(v.position.z));
        });

        Mesh::new(gfx, &vertices, &indicies)
    }
}

fn middle(a: u32, b: u32, cache: &mut HashMap<(u32, u32), u32>, v: &mut Vec<Vertex>) -> u32 {
    let less = a.min(b);
    let more = a.max(b);

    if let Some(v) = cache.get(&(more, less)) {
        return *v;
    }

    let a = v[a as usize];
    let b = v[b as usize];

    let mid = Vertex::new(
        (a.position[0] + b.position[0]) / 2.0,
        (a.position[1] + b.position[1]) / 2.0,
        (a.position[2] + b.position[2]) / 2.0,
    );

    let i = v.len() as u32;
    v.push(mid);

    cache.insert((more, less), i);

    i
}
