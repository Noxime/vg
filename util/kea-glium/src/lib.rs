// ISC License
//
// This is a graphics backend designed for API development, and it uses Glium as
// its own rendering implementation. Before I ship, this will get replaced by
// kea-hal or kea-wgpu, which will be a more low-level implementation to support
// Vulkan, DirectX and Metal enabled devices. Glium only uses OpenGL under the
// hood, so its not as "cool" ;)
// 
// Comments here are quite unclear and hard to read, so I don't recommend
// messing with this file much :P
//
//     Noxim 2019-04-12

pub use glium::glutin;
use glium::implement_vertex;

use kea::renderer::{Color, View, Transform, Scale, Size, Shading};

use std::rc::Rc;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

pub struct Renderer {
    display: Rc<glium::Display>,
    surface: Surface,
    verts: Rc<glium::VertexBuffer<Vertex>>,
    program: Rc<glium::Program>,
}
pub struct Texture {
    tex: glium::texture::Texture2d,
    verts: Rc<glium::VertexBuffer<Vertex>>,
    program: Rc<glium::Program>,
}
pub struct Surface {
    display: Rc<glium::Display>,
    frame: glium::Frame,
    verts: Rc<glium::VertexBuffer<Vertex>>,
    program: Rc<glium::Program>,
}

impl Renderer {
    pub fn new() -> (Renderer, glutin::EventsLoop) {
        println!("GLIUM: new");
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
                    .with_dimensions(glutin::dpi::LogicalSize::new(1280.0, 720.0))
                    .with_title("Kea");
                let context = glutin::ContextBuilder::new()
                    .with_gl(*api);
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

        let vertices = Rc::new(glium::VertexBuffer::new(&*display, &shape).unwrap());

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

        let surface = Surface {
            display: display.clone(),
            frame: display.draw(),
            verts: vertices.clone(),
            program: program.clone(),
        };

        (
            Renderer {
                display,
                surface,
                verts: vertices,
                program,
            },
            events,
        )
    }
}

impl kea::renderer::Renderer for Renderer {
    const NAME: &'static str = "DEV (glium)";

    type Texture = Texture;
    type Surface = Surface;

    fn surface(&mut self) -> &mut Surface {
        &mut self.surface
    }
}

impl kea::renderer::Texture<Renderer> for Texture {
    fn new(renderer: &mut Renderer, size: &Size, color: &Color) -> Self {
        // create a buffer
        let img = (0..size[0] * size[1] * 4).map(|i| color[i % 4]);
        let raw = glium::texture::RawImage2d::from_raw_rgba(
            img.collect(),
            (size[0] as u32, size[1] as u32),
        );
        let tex = glium::texture::Texture2d::new(&*renderer.display, raw).unwrap();

        Texture {
            tex,
            verts: renderer.verts.clone(),
            program: renderer.program.clone(),
        }
    }

    fn from_data(renderer: &mut Renderer, size: &Size, data: &Vec<Color>) -> Self {
        // turn the data into a linear buffer
        let mut img = Vec::new();
        img.reserve(size[0] * size[1] * 4);
        for i in 0..size[0] * size[1] {
            img.push(data[i][0]);
            img.push(data[i][1]);
            img.push(data[i][2]);
            img.push(data[i][3]);
        }
        let raw = glium::texture::RawImage2d::from_raw_rgba(img, (size[0] as u32, size[1] as u32));
        let tex = glium::texture::Texture2d::new(&*renderer.display, raw).unwrap();

        Texture {
            tex,
            verts: renderer.verts.clone(),
            program: renderer.program.clone(),
        }
    }

    fn clone(&self) -> Self {
        // TODO: Read the texture and upload new one
        unimplemented!()
    }
}

impl kea::renderer::Target<Renderer> for Texture {
    fn size(&self) -> Size {
        [
            self.tex.get_width() as _,
            self.tex.get_height().unwrap() as _,
        ]
    }
    fn set(&mut self, color: &Color) {
        use glium::Surface;

        self.tex
            .as_surface()
            .clear_color_srgb(color[0], color[1], color[2], color[3]);
    }
    fn draw(&mut self, texture: &Texture, shading: &Shading, view: &View, transform: &Transform) {
        use glium::uniform;
        use glium::Surface;

        let (ax, ay) = match view.scale {
            Scale::Vertical(s) => (self.size()[0] as f32 / self.size()[1] as f32 * s, s),
            Scale::Horizontal(s) => (s, self.size()[1] as f32 / self.size()[0] as f32 * s),
        };

        let r = transform.rotation - view.rotation;
        let sx = transform.scale_x;
        let sy = transform.scale_y;
        let x = transform.x - view.x;
        let y = transform.y - view.y;

        let tx = texture.size()[0] as f32 / view.pixels_per_unit;
        let ty = texture.size()[1] as f32 / view.pixels_per_unit;

        let vsx = sx / ax * 2.0;
        let vsy = sy / ay * 2.0;

        let uniforms = uniform! {
            add: shading.add,
            multiply: shading.multiply,
            matrix: [
                [r.cos() * vsx * tx, r.sin() * vsy * ty, 0.0, 0.0],
                [-r.sin() * vsx * tx, r.cos() * vsy * ty, 0.0, 0.0],
                [0.0f32, 0.0, 1.0, 0.0],
                [x * vsx, y * vsy, 0.0, 1.0],
            ],
            tex: texture.tex.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        };

        self.tex
            .as_surface()
            .draw(
                &*self.verts,
                &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
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

impl kea::renderer::Surface<Renderer> for Surface {
    fn capture(&self) -> Texture {
        use kea::renderer::Target;
        use glium::Surface;
        let size = self.size();
        let tex = glium::texture::Texture2d::empty(&*self.display, size[0] as u32, size[1] as u32).expect("Glium tex read failure");
        self.frame.blit_whole_color_to(&tex.as_surface(), &glium::BlitTarget {
            left: 0,
            bottom: size[1] as u32,
            width: size[0] as i32,
            height: -(size[1] as i32),
        }, glium::uniforms::MagnifySamplerFilter::Nearest);
        Texture {
            tex,
            program: self.program.clone(),
            verts: self.verts.clone(),
        }

    }

    fn present(&mut self, _vsync: bool) {
        // TODO: vsync
        std::mem::replace(&mut self.frame, self.display.draw())
            .finish()
            .unwrap()
    }
}

impl kea::renderer::Target<Renderer> for Surface {
    fn size(&self) -> Size {
        use glium::Surface;

        let (w, h) = self.frame.get_dimensions();
        [w as usize, h as usize]
    }
    fn set(&mut self, color: &Color) {
        use glium::Surface;

        self.frame
            .clear_color_srgb(color[0], color[1], color[2], color[3]);
    }
    fn draw(&mut self, texture: &Texture, shading: &Shading, view: &View, transform: &Transform) {
        use glium::uniform;
        use glium::Surface;

        let (ax, ay) = match view.scale {
            Scale::Vertical(s) => (self.size()[0] as f32 / self.size()[1] as f32 * s, s),
            Scale::Horizontal(s) => (s, self.size()[1] as f32 / self.size()[0] as f32 * s),
        };

        let r = transform.rotation - view.rotation;
        let sx = transform.scale_x;
        let sy = transform.scale_y;
        let x = transform.x - view.x;
        let y = transform.y - view.y;

        let tx = texture.size()[0] as f32 / view.pixels_per_unit;
        let ty = texture.size()[1] as f32 / view.pixels_per_unit;

        let vsx = sx / ax * 2.0;
        let vsy = sy / ay * 2.0;

        let uniforms = uniform! {
            add: shading.add,
            multiply: shading.multiply,
            matrix: [
                [r.cos() * vsx * tx, r.sin() * vsy * ty, 0.0, 0.0],
                [-r.sin() * vsx * tx, r.cos() * vsy * ty, 0.0, 0.0],
                [0.0f32, 0.0, 1.0, 0.0],
                [x * vsx, y * vsy, 0.0, 1.0],
            ],
            tex: texture.tex.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        };

        self.frame
            .draw(
                &*self.verts,
                &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
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

impl Drop for Surface {
    fn drop(&mut self) {
        self.frame.set_finish().unwrap();
    }
}