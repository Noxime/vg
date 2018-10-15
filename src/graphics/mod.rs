// FIXME: This whole fucking file holy shit what the fuck lmfao someone pls PR
// the fuck out of this shitty fucker what the fucking hell holy fuck shit

#[cfg(feature = "backend-dx")]
extern crate gfx_backend_dx12;
#[cfg(feature = "backend-gl")]
extern crate gfx_backend_gl;
#[cfg(feature = "backend-mt")]
extern crate gfx_backend_metal;
#[cfg(feature = "backend-vk")]
extern crate gfx_backend_vulkan;
extern crate gfx_hal;

use scene::*;
use vectors::*;

#[cfg(feature = "backend-gl")]
use graphics::gfx_backend_gl::glutin::GlContext;
use winit::*;

// pub use self::gfx_hal::command::Buffer;
pub use self::gfx_hal::{
    adapter::*,
    command::*,
    format::*,
    image::*,
    memory::*,
    pass::*,
    pso::*,
    window::*,
    *,
};

use std::{
    mem::size_of,
    sync::Mutex,
};

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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub tex: [f32; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CameraUniformBlock {
    pub projection: [[f32; 4]; 4],
}

pub struct Data<B: Backend> {
    pub size: Vec2<usize>,                      // window size
    pub window: Option<Window>,                 // winit window
    pub queue_group: QueueGroup<B, Graphics>,   // queue group
    pub command_pool: CommandPool<B, Graphics>, // command pool
    pub surface: <B as Backend>::Surface,       // window surface
    pub adapter: Adapter<B>,                    // physical device
    pub device: <B as Backend>::Device,         // logical device
    pub swapchain: Option<<B as Backend>::Swapchain>, // swapchain
    pub frame_views: Vec<<B as Backend>::ImageView>, // frame views
    pub framebuffers: Vec<<B as Backend>::Framebuffer>, // framebuffers
    pub frame_semaphore: <B as Backend>::Semaphore, // frame semaphore
    pub frame_fence: <B as Backend>::Fence,     // frame fence
    pub render_pass: <B as Backend>::RenderPass, // default render pass
    pub pipeline_layout: <B as Backend>::PipelineLayout, // pipeline layout
    pub pipeline: <B as Backend>::GraphicsPipeline, // pipeline
    pub frame_index: u32,                       // frame index
    pub command_buffers: Vec<Submit<B, Graphics, OneShot, Primary>>, // command buffers
    pub format: Format, // framebuffer format
}

pub enum APIData {
    #[cfg(feature = "backend-gl")]
    GL(Data<GLBack>),
    #[cfg(feature = "backend-vk")]
    VK(Data<VKBack>),
    #[cfg(feature = "backend-mt")]
    MT(Data<MTBack>),
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

pub fn create(
    size: Vec2<usize>,
    title: String,
    api: &API,
) -> Result<EventsLoop, RenderError> {
    trace!(
        "Creating {:?} based window with size {}x{}",
        api,
        size.x,
        size.y
    );

    let events = EventsLoop::new();

    let window_builder = WindowBuilder::new()
        .with_dimensions(dpi::LogicalSize::new(size.x as _, size.y as _))
        .with_title(title);

    match api {
        #[cfg(feature = "backend-gl")]
        API::GL => {
            let window = {
                let builder = gfx_backend_gl::config_context(
                    gfx_backend_gl::glutin::ContextBuilder::new(),
                    Rgba8Srgb::SELF,
                    None,
                ).with_vsync(true);
                gfx_backend_gl::glutin::GlWindow::new(
                    window_builder,
                    builder,
                    &events,
                ).unwrap()
            };
            let mut surface = gfx_backend_gl::Surface::from_window(window);
            let mut adapter = pick_adapter(surface.enumerate_adapters())?;
            *API_DATA.lock().unwrap() = Some(APIData::GL(prepare_renderer(
                size, None, adapter, surface,
            )?));
        }
        #[cfg(feature = "backend-vk")]
        API::VK => {
            let window = window_builder.build(&events).unwrap();
            let instance = gfx_backend_vulkan::Instance::create("kea", 1);
            let mut surface = instance.create_surface(&window);
            let mut adapter = pick_adapter(instance.enumerate_adapters())?;
            *API_DATA.lock().unwrap() = Some(APIData::VK(prepare_renderer(
                size,
                Some(window),
                adapter,
                surface,
            )?));
        }
        #[cfg(feature = "backend-mt")]
        API::MT => {
            let window = window_builder.build(&events).unwrap();
            let instance = gfx_backend_metal::Instance::create("kea", 1);
            let mut surface = instance.create_surface(&window);
            let mut adapter = pick_adapter(instance.enumerate_adapters())?;
            *API_DATA.lock().unwrap() = Some(APIData::MT(prepare_renderer(
                size,
                Some(window),
                adapter,
                surface,
            )?));
        }
        #[cfg(feature = "backend-dx")]
        API::DX => {
            let window = window_builder.build(&events).unwrap();
            let instance = gfx_backend_dx12::Instance::create("kea", 1);
            let mut surface = instance.create_surface(&window);
            let mut adapter = pick_adapter(instance.enumerate_adapters())?;
            *API_DATA.lock().unwrap() = Some(APIData::DX(prepare_renderer(
                size,
                Some(window),
                adapter,
                surface,
            )?));
        }
    }

    Ok(events)
}

pub fn resize(size: Vec2<usize>) {
    if let Some(ref mut api_data) = *API_DATA.lock().unwrap() {
        match api_data {
            #[cfg(feature = "backend-gl")]
            APIData::GL(ref mut d) => {
                d.surface.get_window().resize(dpi::PhysicalSize::new(
                    size.x as f64,
                    size.y as f64,
                ));
                _resize(size, d)
            }
            #[cfg(feature = "backend-vk")]
            APIData::VK(ref mut d) => _resize(size, d),
            #[cfg(feature = "backend-mt")]
            APIData::MT(ref mut d) => _resize(size, d),
            #[cfg(feature = "backend-dx")]
            APIData::DX(ref mut d) => _resize(size, d),
        }
    } else {
        error!("No API_DATA in pre_render()! (Create was not called?)");
        panic!()
    }
    // surface
    //     .get_window()
    //     .resize(dims.to_physical(surface.get_window().get_hidpi_factor()));
}

fn _resize<B: Backend>(size: Vec2<usize>, data: &mut Data<B>) {
    debug!("Resizing");
    data.device.wait_idle().unwrap();

    let (caps, formats, _present_modes) = data
        .surface
        .compatibility(&mut data.adapter.physical_device);
    // Verify that previous format still exists so we may resuse it.
    assert!(formats.iter().any(|fs| fs.contains(&data.format)));

    let swap_config = SwapchainConfig::from_caps(&caps, data.format);
    debug!("{:?}", swap_config);
    let extent = swap_config.extent.to_extent();

    // Clean up the old framebuffers, images and swapchain
    for framebuffer in data.framebuffers.split_off(0) {
        data.device.destroy_framebuffer(framebuffer);
    }
    // for (_, rtv) in data.frame_images {
    // data.device.destroy_image_view(rtv);
    // }

    data.device
        .destroy_swapchain(data.swapchain.take().unwrap());

    let (new_swapchain, new_backbuffer) =
        data.device
            .create_swapchain(&mut data.surface, swap_config, None);

    data.swapchain = Some(new_swapchain);

    let color_range = SubresourceRange {
        aspects: Aspects::COLOR,
        levels: 0..1,
        layers: 0..1,
    };

    let (new_frame_images, new_framebuffers) = match new_backbuffer {
        Backbuffer::Images(images) => {
            let pairs = images
                .into_iter()
                .map(|image| {
                    let rtv = data
                        .device
                        .create_image_view(
                            &image,
                            image::ViewKind::D2,
                            data.format,
                            Swizzle::NO,
                            color_range.clone(),
                        ).unwrap();
                    (image, rtv)
                }).collect::<Vec<_>>();
            let fbos = pairs
                .iter()
                .map(|&(_, ref rtv)| {
                    data.device
                        .create_framebuffer(
                            &data.render_pass,
                            Some(rtv),
                            extent,
                        ).unwrap()
                }).collect();
            (pairs, fbos)
        }
        Backbuffer::Framebuffer(fbo) => (Vec::new(), vec![fbo]),
    };

    data.framebuffers = new_framebuffers;
    data.size = size;
    // data.frame_images = new_frame_images;
    // viewport.rect.w = extent.width as _;
    // viewport.rect.h = extent.height as _;
}

pub fn pre_render() {
    if let Some(ref mut api_data) = *API_DATA.lock().unwrap() {
        match api_data {
            #[cfg(feature = "backend-gl")]
            APIData::GL(ref mut d) => _pre_render(d),
            #[cfg(feature = "backend-vk")]
            APIData::VK(ref mut d) => _pre_render(d),
            #[cfg(feature = "backend-mt")]
            APIData::MT(ref mut d) => _pre_render(d),
            #[cfg(feature = "backend-dx")]
            APIData::DX(ref mut d) => _pre_render(d),
        }
    } else {
        error!("No API_DATA in pre_render()! (Create was not called?)");
        panic!()
    }
}

pub fn post_render() {
    if let Some(ref mut api_data) = *API_DATA.lock().unwrap() {
        match api_data {
            #[cfg(feature = "backend-gl")]
            APIData::GL(ref mut d) => _post_render(d),
            #[cfg(feature = "backend-vk")]
            APIData::VK(ref mut d) => _post_render(d),
            #[cfg(feature = "backend-mt")]
            APIData::MT(ref mut d) => _post_render(d),
            #[cfg(feature = "backend-dx")]
            APIData::DX(ref mut d) => _post_render(d),
        }
    } else {
        error!("No API_DATA in post_render()! (Create was not called?)");
        panic!()
    }
}

pub fn render_init(scene: &mut Scene) {
    if let Some(ref mut api_data) = *API_DATA.lock().unwrap() {
        scene.render_init(api_data);
    } else {
        error!("No API_DATA in render_init()! (Create was not called?)");
        panic!()
    }
}

pub fn render(scene: &mut Scene) {
    if let Some(ref mut api_data) = *API_DATA.lock().unwrap() {
        scene.render(api_data);
    } else {
        error!("No API_DATA in render()! (Create was not called?)");
        panic!()
    }
}

pub fn render_destroy(scene: &mut Scene) {
    if let Some(ref mut api_data) = *API_DATA.lock().unwrap() {
        scene.render_destroy(api_data);
    } else {
        error!("No API_DATA in render_destroy()! (Create was not called?)");
        panic!()
    }
}

fn _pre_render<B: Backend>(data: &mut Data<B>) {
    data.device.reset_fence(&data.frame_fence);
    data.command_pool.reset();

    if let Some(ref mut v) = data.swapchain {
        let frame_index = v
            .acquire_image(!0, FrameSync::Semaphore(&data.frame_semaphore))
            .unwrap();

        data.frame_index = frame_index;
    }
    // trace!("Rendering frame idx {}", frame_index,);
}
fn _post_render<B: Backend>(data: &mut Data<B>) {
    // This is what we submit to the command queue. We wait until frame_semaphore
    // is signalled, at which point we know our chosen image is available to draw
    // on.

    let buffers = data.command_buffers.split_off(0);

    // trace!("{} command buffers", buffers.len());
    let submission = Submission::new()
        .wait_on(&[(&data.frame_semaphore, PipelineStage::BOTTOM_OF_PIPE)])
        .submit(buffers);

    // // We submit the submission to one of our command queues, which will signal
    // // frame_fence once rendering is completed.
    data.queue_group.queues[0].submit(submission, Some(&data.frame_fence));

    // We first wait for the rendering to complete...
    data.device.wait_for_fence(&data.frame_fence, !0);

    // ...and then present the image on screen!
    if let Some(ref v) = data.swapchain {
        if let Err(why) =
            v.present(&mut data.queue_group.queues[0], data.frame_index, &[])
        {
            error!("Present failed: {:#?}", why);
        }
    }
}

fn prepare_renderer<B: Backend>(
    size: Vec2<usize>,
    window: Option<Window>,
    adapter: Adapter<B>,
    mut surface: <B as Backend>::Surface,
) -> Result<Data<B>, RenderError> {
    let mut adapter = adapter;
    let (device, queue_group) = adapter
        .open_with::<_, Graphics>(1, |family| {
            surface.supports_queue_family(family)
        }).map_err(|why| {
            error!(
                "Getting graphics queue failed, {}, returning NoGraphics",
                why
            );
            RenderError::NoGraphics
        })?;

    // let memory_types = adapter.physical_device.memory_properties().memory_types;
    debug!(
        "Memory types: {:#?}",
        adapter.physical_device.memory_properties()
    );
    debug!("Device limits: {:#?}", adapter.physical_device.limits());

    let command_pool = device.create_command_pool_typed(
        &queue_group,
        pool::CommandPoolCreateFlags::empty(),
        MAX_BUFFERS,
    );

    let (caps, formats, _present_modes) =
        surface.compatibility(&mut adapter.physical_device);
    debug!("Surface Formats: {:?}", formats);
    let format = formats.map_or(Format::Rgba8Srgb, |formats| {
        formats
            .iter()
            .find(|format| format.base_format().1 == ChannelType::Srgb)
            .map(|format| *format)
            .unwrap_or(formats[0])
    });

    let swap_config = SwapchainConfig::from_caps(&caps, format);
    debug!("Swap config: {:#?}", swap_config);
    let extent = swap_config.extent.to_extent();
    let (swapchain, backbuffer) =
        device.create_swapchain(&mut surface, swap_config, None);

    let frame_semaphore = device.create_semaphore();
    let frame_fence = device.create_fence(false);

    let render_pass = {
        let color_attachment = Attachment {
            format: Some(format),
            samples: 1,
            ops: AttachmentOps::new(
                AttachmentLoadOp::Clear,
                AttachmentStoreOp::Store,
            ),
            stencil_ops: AttachmentOps::DONT_CARE,
            layouts: Layout::Undefined..Layout::Present,
        };

        // A render pass could have multiple subpasses - but we're using one for now.
        let subpass = SubpassDesc {
            colors: &[(0, Layout::ColorAttachmentOptimal)],
            depth_stencil: None,
            inputs: &[],
            resolves: &[],
            preserves: &[],
        };

        // This expresses the dependencies between subpasses. Again, we only have
        // one subpass for now. Future tutorials may go into more detail.
        let dependency = SubpassDependency {
            passes: SubpassRef::External..SubpassRef::Pass(0),
            stages: PipelineStage::COLOR_ATTACHMENT_OUTPUT
                ..PipelineStage::COLOR_ATTACHMENT_OUTPUT,
            accesses: Access::empty()
                ..(Access::COLOR_ATTACHMENT_READ
                    | Access::COLOR_ATTACHMENT_WRITE),
        };

        device.create_render_pass(
            &[color_attachment],
            &[subpass],
            &[dependency],
        )
    };

    // TODO: Move the whole pipeline stuff out of here
    let set_layout = device.create_descriptor_set_layout(
        &[
            pso::DescriptorSetLayoutBinding {
                binding: 0,
                ty: pso::DescriptorType::SampledImage,
                count: 1,
                stage_flags: ShaderStageFlags::FRAGMENT,
                immutable_samplers: false,
            },
            pso::DescriptorSetLayoutBinding {
                binding: 1,
                ty: pso::DescriptorType::Sampler,
                count: 1,
                stage_flags: ShaderStageFlags::FRAGMENT,
                immutable_samplers: false,
            },
            pso::DescriptorSetLayoutBinding {
                binding: 2,
                ty: pso::DescriptorType::UniformBuffer,
                count: 1,
                stage_flags: ShaderStageFlags::VERTEX,
                immutable_samplers: false,
            },
        ],
        &[],
    );

    let pipeline_layout = device.create_pipeline_layout(&Some(set_layout), &[]);

    let vertex_shader_module = {
        let spirv = include_bytes!("../../built_assets/shaders/default.vs.spv");
        device.create_shader_module(spirv).unwrap()
    };

    let fragment_shader_module = {
        let spirv = include_bytes!("../../built_assets/shaders/default.fs.spv");
        device.create_shader_module(spirv).unwrap()
    };

    let pipeline = {
        let vs_entry = EntryPoint::<B> {
            entry: "main",
            module: &vertex_shader_module,
            specialization: Default::default(),
        };

        let fs_entry = EntryPoint::<B> {
            entry: "main",
            module: &fragment_shader_module,
            specialization: Default::default(),
        };

        let shader_entries = GraphicsShaderSet {
            vertex: vs_entry,
            hull: None,
            domain: None,
            geometry: None,
            fragment: Some(fs_entry),
        };

        let subpass = pass::Subpass {
            index: 0,
            main_pass: &render_pass,
        };

        let mut pipeline_desc = GraphicsPipelineDesc::new(
            shader_entries,
            Primitive::TriangleList,
            Rasterizer::FILL,
            &pipeline_layout,
            subpass,
        );

        pipeline_desc
            .blender
            .targets
            .push(ColorBlendDesc(ColorMask::ALL, BlendState::ALPHA));

        pipeline_desc.vertex_buffers.push(VertexBufferDesc {
            binding: 0,
            stride: size_of::<Vertex>() as u32,
            rate: 0,
        });

        pipeline_desc.attributes.push(pso::AttributeDesc {
            location: 0,
            binding: 0,
            element: pso::Element {
                format: Format::Rg32Float,
                offset: 0,
            },
        });
        pipeline_desc.attributes.push(pso::AttributeDesc {
            location: 1,
            binding: 0,
            element: pso::Element {
                format: Format::Rg32Float,
                offset: size_of::<f32>() as u32 * 2,
            },
        });

        device
            .create_graphics_pipeline(&pipeline_desc, None)
            .unwrap()
    };

    let (frame_views, framebuffers) = match backbuffer {
        Backbuffer::Images(images) => {
            let color_range = SubresourceRange {
                aspects: Aspects::COLOR,
                levels: 0..1,
                layers: 0..1,
            };

            let image_views = images
                .iter()
                .map(|image| {
                    device
                        .create_image_view(
                            image,
                            ViewKind::D2,
                            format,
                            Swizzle::NO,
                            color_range.clone(),
                        ).unwrap()
                }).collect::<Vec<_>>();

            let fbos = image_views
                .iter()
                .map(|image_view| {
                    device
                        .create_framebuffer(
                            &render_pass,
                            vec![image_view],
                            extent,
                        ).unwrap()
                }).collect();

            (image_views, fbos)
        }

        // This arm of the branch is currently only used by the OpenGL backend,
        // which supplies an opaque framebuffer for you instead of giving you control
        // over individual images.
        Backbuffer::Framebuffer(fbo) => (vec![], vec![fbo]),
    };

    Ok(Data {
        size,
        window,
        queue_group,
        surface,
        adapter,
        device,
        command_pool,
        swapchain: Some(swapchain),
        frame_views,
        framebuffers,
        frame_semaphore,
        frame_fence,
        render_pass,
        pipeline_layout,
        pipeline,
        frame_index: 0,
        command_buffers: vec![],
        format,
    })
}

fn pick_adapter<B: Backend>(
    adapters: Vec<Adapter<B>>,
) -> Result<Adapter<B>, RenderError> {
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

pub fn destory() {
    if let Some(api_data) = API_DATA.lock().unwrap().take() {
        match api_data {
            #[cfg(feature = "backend-gl")]
            APIData::GL(mut d) => _destroy(d),
            #[cfg(feature = "backend-vk")]
            APIData::VK(mut d) => _destroy(d),
            #[cfg(feature = "backend-mt")]
            APIData::MT(mut d) => _destroy(d),
            #[cfg(feature = "backend-dx")]
            APIData::DX(mut d) => _destroy(d),
        }
    } else {
        warn!("No api data bound, so nothing to destory (ignored)");
    }
}

fn _destroy<B: Backend>(data: Data<B>) {
    // data.device.destroy_buffer()

    data.device.destroy_graphics_pipeline(data.pipeline);
    data.device.destroy_pipeline_layout(data.pipeline_layout);

    for framebuffer in data.framebuffers {
        data.device.destroy_framebuffer(framebuffer);
    }

    for image_view in data.frame_views {
        data.device.destroy_image_view(image_view);
    }

    data.device.destroy_render_pass(data.render_pass);
    data.device.destroy_fence(data.frame_fence);
    data.device.destroy_semaphore(data.frame_semaphore);
    data.device.destroy_swapchain(data.swapchain.unwrap());

    data.device
        .destroy_command_pool(data.command_pool.into_raw());

    if let Some(w) = data.window {
        drop(w);
    }
}
