#[cfg(feature = "backend-dx")]
extern crate gfx_backend_dx12;
#[cfg(feature = "backend-gl")]
extern crate gfx_backend_gl;
#[cfg(feature = "backend-mt")]
extern crate gfx_backend_metal;
#[cfg(feature = "backend-vk")]
extern crate gfx_backend_vulkan;
extern crate gfx_hal as hal;

use vectors::*;

mod error;
pub use self::error::GraphicsError;
mod window;
pub use self::window::*;
mod backend;
pub use self::backend::GfxBackend;
mod adapter;
pub use self::adapter::GfxAdapter;

use graphics::hal::format::AsFormat;
const COLOR_FORMAT: hal::format::Format = hal::format::Rgba8Srgb::SELF;

#[derive(Debug, Copy, Clone)]
pub enum API {
    #[cfg(feature = "backend-gl")]
    GL,
    #[cfg(feature = "backend-vk")]
    VK,
    #[cfg(feature = "backend-mt")]
    MT,
    #[cfg(feature = "backend-dx")]
    DX,
}

enum BackendEnum {
    #[cfg(feature = "backend-gl")]
    GL(GfxBackend<gfx_backend_gl::Backend>),
    #[cfg(feature = "backend-vk")]
    VK(GfxBackend<gfx_backend_vulkan::Backend>),
    #[cfg(feature = "backend-mt")]
    MT(GfxBackend<gfx_backend_metal::Backend>),
    #[cfg(feature = "backend-dx")]
    DX(GfxBackend<gfx_backend_dx12::Backend>),
}

pub struct Renderer {
    backend: BackendEnum,
}

impl Renderer {
    pub fn apis() -> Vec<API> { vec![] }

    pub fn from(win: &mut Window) -> Self {
        trace!("creating renderer from window");

        info!("Available API paths:");
        #[cfg_attr(rustfmt, rustfmt_skip)] {
            #[cfg(feature = "backend-mt")] info!("  Metal"); 
            #[cfg(feature = "backend-dx")] info!("  DirectX 12"); 
            #[cfg(feature = "backend-vk")] info!("  Vulkan"); 
            #[cfg(feature = "backend-gl")] info!("  OpenGL"); 
        }

        let mut backend = None;
        #[cfg_attr(rustfmt, rustfmt_skip)] {
            #[cfg(feature = "backend-mt")] { backend = backend.or_else(|| GfxBackend::new_mt(win).map_err(|e| warn!("MT error: {:?}", e)).ok().map(|b| { info!("Using backend MT: {}", b.info()); BackendEnum::MT(b) })); }
            #[cfg(feature = "backend-dx")] { backend = backend.or_else(|| GfxBackend::new_dx(win).map_err(|e| warn!("DX error: {:?}", e)).ok().map(|b| { info!("Using backend DX: {}", b.info()); BackendEnum::DX(b) })); }
            #[cfg(feature = "backend-vk")] { backend = backend.or_else(|| GfxBackend::new_vk(win).map_err(|e| warn!("VK error: {:?}", e)).ok().map(|b| { info!("Using backend VK: {}", b.info()); BackendEnum::VK(b) })); }
            #[cfg(feature = "backend-gl")] { backend = backend.or_else(|| GfxBackend::new_gl(win).map_err(|e| warn!("GL error: {:?}", e)).ok().map(|b| { info!("Using backend GL: {}", b.info()); BackendEnum::GL(b) })); }
        }

        let backend = match backend {
            Some(v) => v,
            None => {
                error!("No backends available!");
                unimplemented!("notify user on error");
            }
        };

        Self { backend: backend }
    }

    pub fn resize(&mut self, size: Vec2<usize>) {}
}
