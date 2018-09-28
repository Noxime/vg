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

mod draw_call;
mod mesh;
mod shader;
mod texture;
mod vertex;

use scene::*;
use vectors::*;

use winit::*;

use self::gfx_hal::{
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
pub use self::{
    draw_call::*,
    gfx_hal::Backend,
    mesh::*,
    shader::*,
    texture::*,
    vertex::*,
};

use std::{
    collections::HashMap,
    marker::PhantomData,
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

struct Data<B: Backend> {
    size: Vec2<usize>,                               // window size
    window: Option<Window>,                          // winit window
    queue_group: QueueGroup<B, Graphics>,            // queue group
    command_pool: CommandPool<B, Graphics>,          // command pool
    surface: <B as Backend>::Surface,                // window surface
    adapter: Adapter<B>,                             // physical device
    device: <B as Backend>::Device,                  // logical device
    swapchain: <B as Backend>::Swapchain,            // swapchain
    frame_views: Vec<<B as Backend>::ImageView>,     // frame views
    framebuffers: Vec<<B as Backend>::Framebuffer>,  // framebuffers
    frame_semaphore: <B as Backend>::Semaphore,      // frame semaphore
    frame_fence: <B as Backend>::Fence,              // frame fence
    render_pass: <B as Backend>::RenderPass,         // default render pass
    pipeline_layout: <B as Backend>::PipelineLayout, // pipeline layout
    pipeline: <B as Backend>::GraphicsPipeline,      // pipeline
    meshes: HashMap<usize, <B as Backend>::Buffer>,  // vertex buffer cache
}

enum APIData {
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

pub fn upload_mesh(id: usize, vertices: &Vec<Vertex>) {
    if let Some(ref mut api_data) = *API_DATA.lock().unwrap() {
        match api_data {
            #[cfg(feature = "backend-gl")]
            APIData::GL(ref mut d) => _upload_mesh(id, vertices, d),
            #[cfg(feature = "backend-vk")]
            APIData::VK(ref mut d) => _upload_mesh(id, vertices, d),
            #[cfg(feature = "backend-mt")]
            APIData::MT(ref mut d) => _upload_mesh(id, vertices, d),
            #[cfg(feature = "backend-dx")]
            APIData::DX(ref mut d) => _upload_mesh(id, vertices, d),
        }
    } else {
        error!("No API_DATA in render()! (Create was not called?)");
        panic!()
    }
}

fn _upload_mesh<B: Backend>(
    id: usize,
    list_vertices: &Vec<Vertex>,
    data: &mut Data<B>,
) {
    let vertex_stride = size_of::<Vertex>() as u64;
    let buffer_len = list_vertices.len() as u64 * vertex_stride;

    let buffer_unbound = data
        .device
        .create_buffer(buffer_len, buffer::Usage::VERTEX)
        .unwrap();
    let buffer_req = data.device.get_buffer_requirements(&buffer_unbound);
    let upload_type = data
        .adapter
        .physical_device
        .memory_properties()
        .memory_types
        .iter()
        .enumerate()
        .position(|(id, mem_type)| {
            buffer_req.type_mask & (1 << id) != 0 && mem_type
                .properties
                .contains(memory::Properties::CPU_VISIBLE)
        }).unwrap()
        .into();

    let buffer_memory = data
        .device
        .allocate_memory(upload_type, buffer_req.size)
        .unwrap();
    let vertex_buffer = data
        .device
        .bind_buffer_memory(&buffer_memory, 0, buffer_unbound)
        .unwrap();

    // TODO: check transitions: read/write mapping and vertex buffer read
    {
        let mut vertices = data
            .device
            .acquire_mapping_writer::<Vertex>(
                &buffer_memory,
                0..buffer_req.size,
            ).unwrap();
        vertices[0..list_vertices.len()].copy_from_slice(&list_vertices);
        data.device.release_mapping_writer(vertices);
    }

    data.meshes.insert(id, vertex_buffer);
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
    data.device.reset_fence(&data.frame_fence);
    data.command_pool.reset();

    let frame_index = data
        .swapchain
        .acquire_image(!0, FrameSync::Semaphore(&data.frame_semaphore))
        .unwrap();

    let calls = scene.render();

    trace!(
        "Rendering frame idx {} with {} drawcalls per frame",
        frame_index,
        calls.len()
    );

    // change every frame, then we do.
    let finished_command_buffer = {
        let mut command_buffer =
            data.command_pool.acquire_command_buffer(false);

        // Define a rectangle on screen to draw into.
        // In this case, the whole screen.
        let viewport = Viewport {
            rect: Rect {
                x: 0,
                y: 0,
                w: data.size.x as i16,
                h: data.size.y as i16,
            },
            depth: 0.0..1.0,
        };

        command_buffer.set_viewports(0, &[viewport.clone()]);
        command_buffer.set_scissors(0, &[viewport.rect]);
        command_buffer.bind_graphics_pipeline(&data.pipeline);

        {
            let meshes = calls
                .iter()
                .filter(|c| c.mesh.is_some())
                .map(|c| c.mesh.unwrap());
            for m in meshes {
                if let Some(buf) = data.meshes.get(&m.id) {
                    command_buffer.bind_vertex_buffers(0, Some((buf, 0)));
                }
            }
        }

        // command_buffer.bind_vertex_buffers(0, Some((&vertex_buffer, 0)));
        // command_buffer.bind_graphics_descriptor_sets(
        //     &data.pipeline_layout,
        //     0,
        //     Some(&desciptor_set),
        //     &[],
        // ); //TODO

        {
            // Clear the screen and begin the render pass.
            let mut encoder = command_buffer.begin_render_pass_inline(
                &data.render_pass,
                &data.framebuffers[frame_index as usize],
                viewport.rect,
                &[ClearValue::Color(ClearColor::Float([1.0, 0.0, 1.0, 1.0]))],
            );

            encoder.draw(0..12, 0..1);
        }

        // Finish building the command buffer - it's now ready to send to the
        // GPU.
        command_buffer.finish()
    };

    // This is what we submit to the command queue. We wait until frame_semaphore
    // is signalled, at which point we know our chosen image is available to draw
    // on.
    let submission = Submission::new()
        .wait_on(&[(&data.frame_semaphore, PipelineStage::BOTTOM_OF_PIPE)])
        .submit(vec![finished_command_buffer]);

    // We submit the submission to one of our command queues, which will signal
    // frame_fence once rendering is completed.
    data.queue_group.queues[0].submit(submission, Some(&data.frame_fence));

    // We first wait for the rendering to complete...
    data.device.wait_for_fence(&data.frame_fence, !0);

    // ...and then present the image on screen!
    data.swapchain
        .present(&mut data.queue_group.queues[0], frame_index, &[])
        .expect("Present failed");
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

    let frame_semaphore = device.create_semaphore();
    let frame_fence = device.create_fence(false);
    let render_pass = {
        let color_attachment = Attachment {
            format: Some(Rgba8Srgb::SELF),
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

    let pipeline_layout = device.create_pipeline_layout(&[], &[]);

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
                offset: size_of::<Vec3<f32>>() as u32,
            },
        });

        device
            .create_graphics_pipeline(&pipeline_desc, None)
            .unwrap()
    };

    let ((swapchain, backbuffer), extent) =
        create_swapchain(&size, &device, &mut surface);

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
                            Rgba8Srgb::SELF,
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
        swapchain,
        frame_views,
        framebuffers,
        frame_semaphore,
        frame_fence,
        render_pass,
        pipeline_layout,
        pipeline,
        meshes: HashMap::new(),
    })
}

fn create_swapchain<B: Backend>(
    size: &Vec2<usize>,
    device: &impl Device<B>,
    surface: &mut <B as Backend>::Surface,
) -> ((<B as Backend>::Swapchain, Backbuffer<B>), Extent) {
    let swap_config =
        SwapchainConfig::new(size.x as u32, size.y as u32, Rgba8Srgb::SELF, 2);
    let extent = swap_config.extent.to_extent();
    (device.create_swapchain(surface, swap_config, None), extent)
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
    data.device.destroy_swapchain(data.swapchain);


    data.device.destroy_command_pool(data.command_pool.into_raw());

    if let Some(w) = data.window {
        drop(w);
    }
}
