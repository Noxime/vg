use vg::renderer::{Color, Coordinate, Renderer, Size, Surface, Target, Texture};

use winit::EventsLoop;

mod generic;

pub enum HalRenderer {
    Empty(Box<generic::GRenderer<gfx_backend_empty::Backend>>),
    #[cfg(features = "opgl")]
    Gl(Box<generic::GRenderer<gfx_backend_gl::Backend>>),
    #[cfg(features = "vlkn")]
    Vulkan(Box<generic::GRenderer<gfx_backend_vulkan::Backend>>),
    #[cfg(features = "dx11")]
    Dx11(Box<generic::GRenderer<gfx_backend_dx11::Backend>>),
    #[cfg(features = "dx12")]
    Dx12(Box<generic::GRenderer<gfx_backend_dx12::Backend>>),
    #[cfg(features = "metl")]
    Metal(Box<generic::GRenderer<gfx_backend_metal::Backend>>),
}

pub enum HalTexture {
    Empty(Box<generic::GTexture<gfx_backend_empty::Backend>>),
    #[cfg(features = "opgl")]
    Gl(Box<generic::GTexture<gfx_backend_gl::Backend>>),
    #[cfg(features = "vlkn")]
    Vulkan(Box<generic::GTexture<gfx_backend_vulkan::Backend>>),
    #[cfg(features = "dx11")]
    Dx11(Box<generic::GTexture<gfx_backend_dx11::Backend>>),
    #[cfg(features = "dx12")]
    Dx12(Box<generic::GTexture<gfx_backend_dx12::Backend>>),
    #[cfg(features = "metl")]
    Metal(Box<generic::GTexture<gfx_backend_metal::Backend>>),
}

pub enum HalSurface {
    Empty(generic::GSurface<gfx_backend_empty::Backend>),
    #[cfg(features = "opgl")]
    Gl(generic::GSurface<gfx_backend_gl::Backend>),
    #[cfg(features = "vlkn")]
    Vulkan(generic::GSurface<gfx_backend_vulkan::Backend>),
    #[cfg(features = "dx11")]
    Dx11(generic::GSurface<gfx_backend_dx11::Backend>),
    #[cfg(features = "dx12")]
    Dx12(generic::GSurface<gfx_backend_dx12::Backend>),
    #[cfg(features = "metl")]
    Metal(generic::GSurface<gfx_backend_metal::Backend>),
}

impl HalRenderer {
    pub fn new() -> Self {
        HalRenderer::Empty(generic::GRenderer::new().unwrap())
    }
}

impl Renderer for HalRenderer {
    const NAME: &'static str = "HAL";
    type Texture = HalTexture;
    type Surface = HalSurface;
    fn surface(&mut self) -> &mut Self::Surface {
        unimplemented!()
    }
}

impl Texture<HalRenderer> for HalTexture {
    fn new(size: &Size, color: &Color) -> Self {
        unimplemented!()
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
    fn scale(&mut self, size: &Size) {
        unimplemented!()
    }
}

impl Target<HalRenderer> for HalTexture {
    fn size(&self) -> Size {
        unimplemented!()
    }
    fn set(&mut self, color: &Color) {
        unimplemented!()
    }
    fn draw(&mut self, texture: HalTexture, coords: Coordinate) {
        unimplemented!()
    }
}

impl Surface<HalRenderer> for HalSurface {
    fn capture(&self) -> HalTexture {
        unimplemented!()
    }
}
impl Target<HalRenderer> for HalSurface {
    fn size(&self) -> Size {
        unimplemented!()
    }
    fn set(&mut self, color: &Color) {
        unimplemented!()
    }
    fn draw(&mut self, texture: HalTexture, coords: Coordinate) {
        unimplemented!()
    }
}
