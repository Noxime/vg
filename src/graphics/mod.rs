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
mod device;
pub use self::device::GfxDevice;

use std::{cell::RefCell, rc::Rc};

use graphics::hal::format::AsFormat;
const COLOR_FORMAT: hal::format::Format = hal::format::Rgba8Srgb::SELF;

#[derive(Debug, Copy, Clone)]
#[cfg_attr(rustfmt, rustfmt_skip)]
pub enum API {
    #[cfg(feature = "backend-gl")] GL,
    #[cfg(feature = "backend-vk")] VK,
    #[cfg(feature = "backend-mt")] MT,
    #[cfg(feature = "backend-dx")] DX,
}

#[cfg_attr(rustfmt, rustfmt_skip)] 
pub enum RenderSwitch {
    #[cfg(feature = "backend-gl")] GL(RenderData<gfx_backend_gl::Backend>),
    #[cfg(feature = "backend-vk")] VK(RenderData<gfx_backend_vulkan::Backend>),
    #[cfg(feature = "backend-mt")] MT(RenderData<gfx_backend_metal::Backend>),
    #[cfg(feature = "backend-dx")] DX(RenderData<gfx_backend_dx12::Backend>),
}

pub struct RenderData<B: hal::Backend> {
    backend: GfxBackend<B>,
}

pub struct Renderer {
    data: RenderSwitch,
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

        #[cfg(feature = "backend-mt")]
        {
            match GfxBackend::new_mt(win) {
                Ok(b) => {
                    return Self {
                        data: RenderSwitch::MT(Self::prepare(b)),
                    }
                }
                Err(e) => debug!("Not using backend MT: {:?}", e),
            }
        }
        #[cfg(feature = "backend-dx")]
        {
            match GfxBackend::new_dx(win) {
                Ok(b) => {
                    return Self {
                        data: RenderSwitch::DX(Self::prepare(b)),
                    }
                }
                Err(e) => debug!("Not using backend DX: {:?}", e),
            }
        }
        #[cfg(feature = "backend-vk")]
        {
            match GfxBackend::new_vk(win) {
                Ok(b) => {
                    return Self {
                        data: RenderSwitch::VK(Self::prepare(b)),
                    }
                }
                Err(e) => debug!("Not using backend VK: {:?}", e),
            }
        }
        #[cfg(feature = "backend-gl")]
        {
            match GfxBackend::new_gl(win) {
                Ok(b) => {
                    return Self {
                        data: RenderSwitch::GL(Self::prepare(b)),
                    }
                }
                Err(e) => debug!("Not using backend GL: {:?}", e),
            }
        }

        error!("No backends available!");
        unimplemented!("present user with error");
    }

    fn prepare<B: hal::Backend>(mut backend: GfxBackend<B>) -> RenderData<B> {
        let device = Rc::new(RefCell::new(GfxDevice::new(
            backend.adapter.adapter.take().expect("Adapter gone"),
            &backend.surface,
        )));
        RenderData { backend }
    }

    pub fn resize(&mut self, size: Vec2<usize>) {}
}
