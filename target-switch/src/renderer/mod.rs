use kea::renderer::*;

use libnx_rs::libnx;


pub struct SwitchRenderer {
    surface: SwitchSurface
}
pub struct SwitchTexture {
    width: u32,
    height: u32,
    data: Vec<Color>,
}
pub struct SwitchSurface {
    buf: SwitchTexture,
    fb: *mut [u8; 4]
}

impl SwitchRenderer {
    pub fn new() -> SwitchRenderer {
        let (width, height, fb) = unsafe {
            libnx::gfxInitDefault();
            let mut width = 0u32;
            let mut height = 0u32;
            let fb = libnx::gfxGetFramebuffer(&mut width as *mut u32, &mut height as *mut u32) as *mut u32 as *mut [u8; 4];
            (width, height, fb)
        };

        let mut data = Vec::new();
        data.resize(width as usize * height as usize, [0.0, 0.0, 0.0, 1.0]);

        SwitchRenderer {
            surface: SwitchSurface {
                buf: SwitchTexture {
                    width,
                    height,
                    data
                },
                fb,
            }
        }
    }
}

impl Drop for SwitchRenderer {
    fn drop(&mut self) {
        unsafe {
            libnx::gfxExit();
        }
    }
}


impl Renderer for SwitchRenderer {
    const NAME: &'static str = "Piston (GL32)";
    type Texture = SwitchTexture;
    type Surface = SwitchSurface;

    fn surface(&mut self) -> &mut Self::Surface {
        &mut self.surface
    }
}

impl kea::renderer::Texture<SwitchRenderer> for SwitchTexture {
    fn new(renderer: &mut SwitchRenderer, size: &Size, color: &Color) -> Self {
        let mut data = Vec::new();
        data.resize(size[0] * size[1], *color);
        SwitchTexture {
            width: size[0] as u32,
            height: size[1] as u32,
            data
        }
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl kea::renderer::Target<SwitchRenderer> for SwitchTexture {
    fn size(&self) -> Size {
        [self.width as usize, self.height as usize]
    }
    fn set(&mut self, color: &Color) {
        for i in 0 .. self.width as usize * self.height as usize {
            self.data[i] = *color;
        }
    }
    fn draw(&mut self, _texture: &SwitchTexture, trans: &Matrix) {
        let mut vert0 = multiply(trans, [-0.5, -0.5]);
        let mut vert1 = multiply(trans, [-0.5,  0.5]);
        let mut vert2 = multiply(trans, [ 0.5,  0.5]);
        let mut vert3 = multiply(trans, [ 0.5, -0.5]);

        vert0[0] = (vert0[0] * 0.5 + 0.5) * self.width as f32;
        vert0[1] = (vert0[1] * 0.5 + 0.5) * self.height as f32;
        vert1[0] = (vert1[0] * 0.5 + 0.5) * self.width as f32;
        vert1[1] = (vert1[1] * 0.5 + 0.5) * self.height as f32;
        vert2[0] = (vert2[0] * 0.5 + 0.5) * self.width as f32;
        vert2[1] = (vert2[1] * 0.5 + 0.5) * self.height as f32;
        vert3[0] = (vert3[0] * 0.5 + 0.5) * self.width as f32;
        vert3[1] = (vert3[1] * 0.5 + 0.5) * self.height as f32;

        struct Vertex {
            x: isize,
            y: isize,
            u: f32,
            v: f32,
        }

        let mut verts = [
            Vertex {
                x: 0.max((vert0[0] as isize).min(self.width as isize)),
                y: 0.max((vert0[1] as isize).min(self.height as isize)),
                u: 0.0,
                v: 0.0,
            }, Vertex {
                x: 0.max((vert1[0] as isize).min(self.width as isize)),
                y: 0.max((vert1[1] as isize).min(self.height as isize)),
                u: 0.0,
                v: 1.0,
            }, Vertex {
                x: 0.max((vert2[0] as isize).min(self.width as isize)),
                y: 0.max((vert2[1] as isize).min(self.height as isize)),
                u: 1.0,
                v: 1.0,
            }, Vertex {
                x: 0.max((vert3[0] as isize).min(self.width as isize)),
                y: 0.max((vert3[1] as isize).min(self.height as isize)),
                u: 1.0,
                v: 0.0,
            },
        ];

        verts.sort_by(|a, b| a.y.cmp(&b.y))
    }
}

impl kea::renderer::Surface<SwitchRenderer> for SwitchSurface {
    fn capture(&self) -> SwitchTexture {
        self.buf.clone()
    }
    fn present(&mut self, vsync: bool) {

        // copy internal texture into the switches framebuffer
        for x in 0 .. self.buf.width {
            for y in 0 .. self.buf.height {
                let i = x as isize + y as isize * self.buf.width as isize;
                unsafe {
                    *self.fb.offset(i) = [
                        (self.buf.data[i as usize][0] * 255.0) as u8,
                        (self.buf.data[i as usize][1] * 255.0) as u8,
                        (self.buf.data[i as usize][2] * 255.0) as u8,
                        (self.buf.data[i as usize][3] * 255.0) as u8,
                    ]
                }
            }
        }

        unsafe {
            libnx::gfxFlushBuffers();
            libnx::gfxSwapBuffers();
        }
    }
}

impl kea::renderer::Target<SwitchRenderer> for SwitchSurface {
    fn size(&self) -> Size {
        self.buf.size()
    }

    fn set(&mut self, color: &Color) {
        self.buf.set(color)
    }

    fn draw(&mut self, texture: &SwitchTexture, trans: &Matrix) {
        self.buf.draw(texture, trans)
    }
}

fn multiply(m: &Matrix, v: [f32; 2]) -> [f32; 2] {
    [
        v[0] * m.raw()[0][0] + v[1] * m.raw()[0][1] + m.raw()[0][2],
        v[0] * m.raw()[1][0] + v[1] * m.raw()[1][1] + m.raw()[1][2],
    ]
}