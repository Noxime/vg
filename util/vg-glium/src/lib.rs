// ISC License
//
// This is a graphics backend designed for API development, and it uses Glium as
// its own rendering implementation. Before I ship, this will get replaced by
// vg-hal or vg-wgpu, which will be a more low-level implementation to support
// Vulkan, DirectX and Metal enabled devices. Glium only uses OpenGL under the
// hood, so its not as "cool" ;)
//
// Comments here are quite unclear and hard to read, so I don't recommend
// messing with this file much :P
//
//     Noxim 2019-04-12

pub use glium::glutin;
use glium::implement_vertex;

use vg::{Color, Matrix, Size};

use std::rc::Rc;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

pub struct Gfx {
    display: Rc<glium::Display>,
    frame: glium::Frame,
    verts: Rc<glium::VertexBuffer<Vertex>>,
    program: Rc<glium::Program>,
}
pub struct Texture {
    tex: glium::texture::Texture2d,
    verts: Rc<glium::VertexBuffer<Vertex>>,
    program: Rc<glium::Program>,
}

impl Gfx {
    pub fn new() -> (Self, glutin::EventsLoop) {
        // Iterate through various GL versions to find a compatible one
        let (display, events) = {
            let mut res = None;
            for api in &[
                glutin::GlRequest::Latest,
                glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 0)),
                glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (3, 0)),
                glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (2, 0)),
                glutin::GlRequest::Specific(glutin::Api::WebGl, (1, 0)),
            ] {
                let events = glutin::EventsLoop::new();
                let window = glutin::WindowBuilder::new()
                    .with_dimensions(glutin::dpi::LogicalSize::new(
                        1280.0, 720.0,
                    ))
                    .with_title("vg");
                let context = glutin::ContextBuilder::new().with_gl(*api);
                if let Ok(d) = glium::Display::new(window, context, &events) {
                    println!("Renderer: {}", d.get_opengl_version_string());
                    res = Some((Rc::new(d), events));
                    break;
                }
            }

            res.expect("No graphics api support found")
        };

        // upload a vertex quad for our rendering ops
        let vertex1 = Vertex {
            position: [0.0, 0.0],
            tex_coords: [0.0, 0.0],
        };
        let vertex2 = Vertex {
            position: [0.0, 1.0],
            tex_coords: [0.0, 1.0],
        };
        let vertex3 = Vertex {
            position: [1.0, 1.0],
            tex_coords: [1.0, 1.0],
        };
        let vertex4 = Vertex {
            position: [1.0, 0.0],
            tex_coords: [1.0, 0.0],
        };
        let shape = vec![vertex1, vertex2, vertex3, vertex3, vertex1, vertex4];

        let vertices =
            Rc::new(glium::VertexBuffer::new(&*display, &shape).unwrap());

        // default "identity" shaders
        let program = Rc::new(
            glium::Program::from_source(
                &*display,
                include_str!("shader.vs"),
                include_str!("shader.fs"),
                None,
            )
            .unwrap(),
        );

        (
            Self {
                frame: display.draw(),
                display,
                verts: vertices,
                program,
            },
            events,
        )
    }
}

#[vg::async_trait(?Send)]
impl vg::gfx::GfxTrait for Gfx {
    async fn present(&mut self) {
        // TODO: vsync
        std::mem::replace(&mut self.frame, self.display.draw())
            .finish()
            .unwrap();
    }

    fn backends(&self) -> Vec<vg::gfx::Backend> {
        vec![]
    }
    async fn change_backend(&mut self, backend: usize) -> Result<(), ()> {
        Ok(())
    }

    /// Set vsync state
    fn vsync(&mut self, vsync: bool) {}

    /// Upload the texture to GPU
    async fn texture(
        &mut self,
        mut source: Box<dyn vg::gfx::texture::Source>,
    ) -> Box<dyn vg::gfx::texture::TextureTrait> {
        let (size, data) = source.load().await;
        let d = vec![vec![(0.0f32, 0.0f32, 0.0f32, 0.0f32); size[1]]; size[0]];

        Box::new(Texture {
            tex: glium::texture::Texture2d::new(&*self.display, d).unwrap(),
            program: self.program.clone(),
            verts: self.verts.clone(),
        })
    }
}

impl vg::gfx::texture::TextureTrait for Texture {}

impl vg::gfx::Target for Texture {
    fn size(&self) -> Size {
        [
            self.tex.get_width() as _,
            self.tex.get_height().unwrap() as _,
        ]
    }

    fn fill(&mut self, color: Color) {
        use glium::Surface;

        self.tex
            .as_surface()
            .clear_color_srgb(color[0], color[1], color[2], color[3]);
    }

    fn draw(
        &mut self,
        texture: &vg::gfx::texture::Texture,
        matrices: &[Matrix],
    ) {
        use glium::uniform;
        use glium::Surface;

        for mat in matrices {
            let mat = [
                [mat[0], mat[1], mat[2], mat[3]],
                [mat[4], mat[5], mat[6], mat[7]],
                [mat[8], mat[9], mat[10], mat[11]],
                [mat[12], mat[13], mat[14], mat[15]],
            ];

            // let tex = (texture.tex as Box<dyn std::any::Any+'static>).downcast();
            let tex = <Box::<dyn std::any::Any+'static>>::downcast(texture.tex);

            let uniforms = uniform! {
                matrix: mat,
                tex: texture.tex.sampled().magnify_filter(match texture.sampling() {
                    vg::gfx::texture::Sampling::Nearest => glium::uniforms::MagnifySamplerFilter::Nearest,
                    vg::gfx::texture::Sampling::Linear => glium::uniforms::MagnifySamplerFilter::Linear,
                })
            };
            self.tex
                .as_surface()
                .draw(
                    &*self.verts,
                    &glium::index::NoIndices(
                        glium::index::PrimitiveType::TrianglesList,
                    ),
                    &*self.program,
                    &uniforms,
                    &glium::DrawParameters {
                        blend: glium::Blend::alpha_blending(),
                        ..Default::default()
                    },
                )
                .unwrap();
        }
    }
}

impl vg::gfx::Target for Gfx {
    fn size(&self) -> Size {
        use glium::Surface;

        let (w, h) = self.frame.get_dimensions();
        [w as usize, h as usize]
    }

    fn fill(&mut self, color: Color) {
        use glium::Surface;

        self.frame
            .clear_color_srgb(color[0], color[1], color[2], color[3]);
    }

    fn draw(
        &mut self,
        texture: &vg::gfx::texture::Texture,
        matrices: &[Matrix],
    ) {
        //     use glium::uniform;
        //     use glium::Surface;

        //     for mat in matrices {
        //         let uniforms = uniform! {
        //             matrix: *mat,
        //             tex:
        // texture.tex.downcast::<Texture>().unwrap().sampled().
        // magnify_filter(match texture.sampling() {                 
        // vg::gfx::texture::Sampling::Nearest =>
        // glium::uniforms::MagnifySamplerFilter::Nearest,              
        // vg::gfx::texture::Sampling::Linear =>
        // glium::uniforms::MagnifySamplerFilter::Linear,             })
        //         };
        //         self.frame
        //             .draw(
        //                 &*self.verts,
        //                 &glium::index::NoIndices(
        //                     glium::index::PrimitiveType::TrianglesList,
        //                 ),
        //                 &*self.program,
        //                 &uniforms,
        //                 &glium::DrawParameters {
        //                     blend: glium::Blend::alpha_blending(),
        //                     ..Default::default()
        //                 },
        //             )
        //             .unwrap();
        //     }
        // }
    }
}
