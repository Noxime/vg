extern crate gfx_hal;
#[cfg(feature = "backend-gl")] extern crate gfx_backend_gl;
#[cfg(feature = "backend-vk")] extern crate gfx_backend_vulkan;
#[cfg(feature = "backend-mt")] extern crate gfx_backend_metal;
#[cfg(feature = "backend-dx")] extern crate gfx_backend_dx12;
extern crate winit;

use vectors::*;

use self::gfx_hal::format::{AsFormat, Rgba8Srgb as ColorFormat};
use self::gfx_hal::{Instance, Surface, Adapter};

#[derive(Debug, Copy, Clone)]
pub enum API {
    #[cfg(feature = "backend-gl")] GL,
    #[cfg(feature = "backend-vk")] VK,
    #[cfg(feature = "backend-mt")] MT,
    #[cfg(feature = "backend-dx")] DX,
}

pub fn supported() -> Vec<(API, bool)> {
    vec![
        #[cfg(feature = "backend-gl")] (API::GL, true),
        #[cfg(feature = "backend-vk")] (API::VK, true),
        #[cfg(feature = "backend-mt")] (API::MT, true),
        #[cfg(feature = "backend-dx")] (API::DX, true),
    ]
}

pub fn create(size: Vec2<usize>, title: String, api: &API) {
    trace!("Creating {:?} based window with size {}x{}", api, size.x, size.y);
    let mut events = winit::EventsLoop::new();

    let window_builder = winit::WindowBuilder::new()
        .with_dimensions(winit::dpi::LogicalSize::new(
            size.x as _,
            size.y as _,
        ))
        .with_title(title);

    let (mut adapters, mut surface) = match api {
        #[cfg(feature = "backend-gl")] API::GL => {
            let window = {
                let builder = gfx_backend_gl::config_context(gfx_backend_gl::glutin::ContextBuilder::new(), ColorFormat::SELF, None)
                    .with_vsync(true);
                gfx_backend_gl::glutin::GlWindow::new(window_builder, builder, &events).unwrap()
            };
            let surface = gfx_backend_gl::Surface::from_window(window);
            let adapters = surface.enumerate_adapters();
            (adapters, surface)
        },
        #[cfg(any(feature = "backend-vk", feature = "backend-mt", feature = "backend-dx"))]
        api => {
            let window = window_builder.build(&events).unwrap();
            let instance = match api {
                #[cfg(feature = "backend-vk")] API::VK => gfx_backend_vulkan::Instance::create("KEA", 1),
                #[cfg(feature = "backend-mt")] API::MT => gfx_backend_metal::Instance::create("KEA", 1),
                #[cfg(feature = "backend-dx")] API::DX => gfx_backend_dx12::Instance::create("KEA", 1),
            };
            let surface = instance.create_surface(&window);
            let adapters = instance.enumerate_adapters();
            (adapters, surface)
        },
    };

}