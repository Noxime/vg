#[cfg(feature = "backend-dx")]
extern crate gfx_backend_dx12;
#[cfg(feature = "backend-gl")]
extern crate gfx_backend_gl;
#[cfg(feature = "backend-mt")]
extern crate gfx_backend_metal;
#[cfg(feature = "backend-vk")]
extern crate gfx_backend_vulkan;
extern crate gfx_hal;

use vectors::*;
use winit::*;

use self::gfx_hal::format::{AsFormat, Rgba8Srgb as ColorFormat};
use self::gfx_hal::{
    adapter::DeviceType, image, pass, pool, pso::PipelineStage, Adapter, Backend, Device, Graphics,
    Instance, PhysicalDevice, Surface,
};

use std::sync::{Arc, Mutex};

const MAX_BUFFERS: usize = 16;

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

/// Various errors that might rise from graphics api invocations
#[derive(Debug)]
pub enum RenderError {
    /// There were no devices available for given api
    NoAdapter,
    /// This adapter and api do not support graphics
    NoGraphics,
}

// return the support status for available API's in the preferred order
pub fn supported() -> Vec<(API, bool)> {
    vec![
        #[cfg(feature = "backend-mt")]
        (API::MT, true),
        #[cfg(feature = "backend-vk")]
        (API::VK, true),
        #[cfg(feature = "backend-gl")]
        (API::GL, true),
        #[cfg(feature = "backend-dx")]
        (API::DX, true),
    ]
}

// lazy_static! {
//     static ref BACKEND: Backend
//

pub fn create(size: Vec2<usize>, title: String, api: &API) -> Result<EventsLoop, RenderError> {
    trace!(
        "Creating {:?} based window with size {}x{}",
        api,
        size.x,
        size.y
    );
    let mut events = EventsLoop::new();

    let window_builder = WindowBuilder::new()
        .with_dimensions(dpi::LogicalSize::new(size.x as _, size.y as _))
        .with_title(title);

    // let mut adapters: Vec<Adapter<Backend=B>>;

    match api {
        #[cfg(feature = "backend-gl")]
        API::GL => {
            let window = {
                let builder = gfx_backend_gl::config_context(
                    gfx_backend_gl::glutin::ContextBuilder::new(),
                    ColorFormat::SELF,
                    None,
                ).with_vsync(true);
                gfx_backend_gl::glutin::GlWindow::new(window_builder, builder, &events).unwrap()
            };
            let surface = gfx_backend_gl::Surface::from_window(window);
            let mut adapter = pick_adapter(surface.enumerate_adapters())?;
            prepare_renderer(adapter, surface)?
        }
        #[cfg(feature = "backend-vk")]
        API::VK => {
            let window = window_builder.build(&events).unwrap();
            let instance = gfx_backend_vulkan::Instance::create("kea", 1);
            let surface = instance.create_surface(&window);
            let mut adapter = pick_adapter(instance.enumerate_adapters())?;
            prepare_renderer(adapter, surface)?
        }
        #[cfg(feature = "backend-mt")]
        API::MT => {
            let window = window_builder.build(&events).unwrap();
            let instance = gfx_backend_metal::Instance::create("kea", 1);
            let surface = instance.create_surface(&window);
            let mut adapter = pick_adapter(instance.enumerate_adapters())?;
            prepare_renderer(adapter, surface)?
        }
        #[cfg(feature = "backend-dx")]
        API::DX => {
            let window = window_builder.build(&events).unwrap();
            let instance = gfx_backend_dx12::Instance::create("kea", 1);
            let surface = instance.create_surface(&window);
            let mut adapter = pick_adapter(instance.enumerate_adapters())?;
            prepare_renderer(adapter, surface)?
        }
    }

    Ok(events)
}

fn prepare_renderer<B: Backend>(
    adapter: Adapter<B>,
    surface: impl Surface<B>,
) -> Result<(), RenderError> {
    let mut adapter = adapter;
    let (device, mut queue_group) = adapter
        .open_with::<_, Graphics>(1, |family| surface.supports_queue_family(family))
        .map_err(|why| {
            error!(
                "Getting graphics queue failed, {}, returning NoGraphics",
                why
            );
            RenderError::NoGraphics
        })?;
    let mut command_pool = device.create_command_pool_typed(
        &queue_group,
        pool::CommandPoolCreateFlags::empty(),
        MAX_BUFFERS,
    );
    let render_pass = create_render_pass(device);
    Ok(())
}

fn pick_adapter<B: Backend>(adapters: Vec<Adapter<B>>) -> Result<Adapter<B>, RenderError> {
    if adapters.len() == 0 {
        return Err(RenderError::NoAdapter);
    }
    let mut adapters = adapters;
    debug!("Adapters available:");
    for adapter in &adapters {
        debug!(
            "  {}:{} {} \"{}\"",
            adapter.info.vendor,
            adapter.info.device,
            match adapter.info.device_type {
                DeviceType::Other => "unknown",
                DeviceType::IntegratedGpu => "integrated",
                DeviceType::DiscreteGpu => "discrete",
                DeviceType::VirtualGpu => "virtual",
                DeviceType::Cpu => "software",
            },
            adapter.info.name
        );
    }
    Ok(adapters.remove(0))
}

fn create_render_pass<B: Backend>(device: impl Device<B>) -> <B as Backend>::RenderPass {
    let attachment = pass::Attachment {
        format: Some(ColorFormat::SELF),
        samples: 1,
        ops: pass::AttachmentOps::new(
            pass::AttachmentLoadOp::Clear,
            pass::AttachmentStoreOp::Store,
        ),
        stencil_ops: pass::AttachmentOps::DONT_CARE,
        layouts: image::Layout::Undefined..image::Layout::Present,
    };

    let subpass = pass::SubpassDesc {
        colors: &[(0, image::Layout::ColorAttachmentOptimal)],
        depth_stencil: None,
        inputs: &[],
        resolves: &[],
        preserves: &[],
    };

    let dependency = pass::SubpassDependency {
        passes: pass::SubpassRef::External..pass::SubpassRef::Pass(0),
        stages: PipelineStage::COLOR_ATTACHMENT_OUTPUT..PipelineStage::COLOR_ATTACHMENT_OUTPUT,
        accesses: image::Access::empty()
            ..(image::Access::COLOR_ATTACHMENT_READ | image::Access::COLOR_ATTACHMENT_WRITE),
    };

    device.create_render_pass(&[attachment], &[subpass], &[dependency])
}
