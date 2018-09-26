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
use scene::*;

use winit::*;

use self::gfx_hal::format::{AsFormat, Rgba8Srgb as ColorFormat};
use self::gfx_hal::*;
use self::gfx_hal::adapter::*;
use self::gfx_hal::pso::*;
use self::gfx_hal::window::*;
pub use self::gfx_hal::Backend;

use std::sync::{Arc, Mutex};

const MAX_BUFFERS: usize = 16;

lazy_static! {
    static ref API_DATA: Mutex<Option<APIData>> = Mutex::new(None); 
}

#[cfg(feature = "backend-gl")]
pub type GLBack = gfx_backend_gl::Backend;
#[cfg(feature = "backend-vk")]
pub type VKBack = gfx_backend_vulkan::Backend;
#[cfg(feature = "backend-mt")]
pub type MTBack = gfx_backend_metal::Backend;
#[cfg(feature = "backend-dx")]
pub type DXBack = gfx_backend_dx12::Backend;

struct Data<B: Backend> {
    swapchain: <B as Backend>::Swapchain,
    frame_semaphore: <B as Backend>::Semaphore,
}
enum APIData {
    #[cfg(feature = "backend-gl")]
    GL(Data<GLBack>),
    #[cfg(feature = "backend-vk")]
    VK(Data<VKBack>),
    #[cfg(feature = "backend-mt")]
    VK(Data<MTBack>),
    #[cfg(feature = "backend-dx")]
    DX(Data<DXBack>),
}

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
            let mut surface = gfx_backend_gl::Surface::from_window(window);
            let mut adapter = pick_adapter(surface.enumerate_adapters())?;
            let (swapchain, frame_semaphore) = prepare_renderer(size, adapter, &mut surface)?;
            let mut data = API_DATA.lock().unwrap();
            *data = Some(APIData::GL(Data {
                swapchain,
                frame_semaphore,
            }));
        }
        #[cfg(feature = "backend-vk")]
        API::VK => {
            let window = window_builder.build(&events).unwrap();
            let instance = gfx_backend_vulkan::Instance::create("kea", 1);
            let mut surface = instance.create_surface(&window);
            let mut adapter = pick_adapter(instance.enumerate_adapters())?;
            let (swapchain, frame_semaphore) = prepare_renderer(size, adapter, &mut surface)?;
            let mut data = API_DATA.lock().unwrap();
            *data = Some(APIData::GL(Data {
                swapchain,
                frame_semaphore,
            }));
        }
        #[cfg(feature = "backend-mt")]
        API::MT => {
            let window = window_builder.build(&events).unwrap();
            let instance = gfx_backend_metal::Instance::create("kea", 1);
            let mut surface = instance.create_surface(&window);
            let mut adapter = pick_adapter(instance.enumerate_adapters())?;
            let (swapchain, frame_semaphore) = prepare_renderer(size, adapter, &mut surface)?;
            let mut data = API_DATA.lock().unwrap();
            *data = Some(APIData::GL(Data {
                swapchain,
                frame_semaphore,
            }));
        }
        #[cfg(feature = "backend-dx")]
        API::DX => {
            let window = window_builder.build(&events).unwrap();
            let instance = gfx_backend_dx12::Instance::create("kea", 1);
            let mut surface = instance.create_surface(&window);
            let mut adapter = pick_adapter(instance.enumerate_adapters())?;
            let (swapchain, frame_semaphore) = prepare_renderer(size, adapter, &mut surface)?;
            let mut data = API_DATA.lock().unwrap();
            *data = Some(APIData::GL(Data {
                swapchain,
                frame_semaphore,
            }));
        }
    }

    Ok(events)
}

pub fn render(scene: &mut Scene) {
    if let Some(ref mut api_data) = *API_DATA.lock().unwrap() {
        match api_data {
            #[cfg(feature = "backend-gl")]
            APIData::GL(ref mut d) => _render(scene, d),
            #[cfg(feature = "backend-vk")]
            APIData::VK(ref mut d) => _render(scene, d),
            #[cfg(feature = "backend-mt")]
            APIData::MT(ref mut d) => _render(scene, d),
            #[cfg(feature = "backend-dx")]
            APIData::DX(ref mut d) => _render(scene, d),
        }
    } else {
        error!("No API_DATA in render()! (Create was not called?)");
        panic!()
    }
}

fn _render<B: Backend>(scene: &mut Scene, data: &mut Data<B>) {
    let frame_index = data.swapchain.acquire_image(0, FrameSync::Semaphore(&data.frame_semaphore)).unwrap();

    scene.render();
}

fn prepare_renderer<B: Backend>(
    size: Vec2<usize>,
    adapter: Adapter<B>,
    surface: &mut <B as Backend>::Surface,
) -> Result<(
    <B as Backend>::Swapchain, 
    <B as Backend>::Semaphore
), RenderError> {
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
    let render_pass = create_render_pass(&device);
    let (mut swapchain, backbuffer) = create_swapchain(size, &device, surface);
    let frame_semaphore = device.create_semaphore();
    let frame_fence = device.create_fence(false);
    Ok((swapchain, frame_semaphore))
}

fn create_swapchain<B: Backend>(size: Vec2<usize>, device: &impl Device<B>, surface: &mut <B as Backend>::Surface) -> (<B as Backend>::Swapchain, Backbuffer<B>) {
    let swap_config = SwapchainConfig::new(size.x as u32, size.y as u32, ColorFormat::SELF, 2);
    device.create_swapchain(surface, swap_config, None)
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

fn create_render_pass<B: Backend>(device: &impl Device<B>) -> <B as Backend>::RenderPass {
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
