use std::{sync::Arc, time::Instant};

use log::*;
use rend3::{
    datatypes::{AffineTransform, Mesh, Object},
    CustomDevice, Renderer, RendererBuilder, RendererOptions, RendererOutput, VSyncMode,
};
use rend3_list::{DefaultPipelines, DefaultShaders};
use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};

const PREFERRED_FORMAT: TextureFormat = TextureFormat::Bgra8Unorm;
// const PREFERRED_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
// const FALLBACK_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;

pub struct Gfx {
    instance: Arc<Instance>,
    surface: Surface,
    device: Arc<Device>,
    queue: Arc<Queue>,
    window: Arc<Window>,
    swapchain_desc: SwapChainDescriptor,
    swapchain: SwapChain,
    renderer: Arc<Renderer>,
    pipelines: DefaultPipelines,
    #[cfg(feature = "debug")]
    pub egui_pass: egui_wgpu_backend::RenderPass,
}

impl Gfx {
    pub async fn new(window: Arc<Window>) -> Gfx {
        let backends = BackendBit::all();
        debug!("Using graphics backends: {:?}", backends);

        let instance = Instance::new(backends);
        let surface = unsafe { instance.create_surface(window.as_ref()) };

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Could not find suitable adapter");

        debug!("Using: {}", adapter.get_info().name);
        for adapter in instance.enumerate_adapters(backends) {
            let info = adapter.get_info();
            debug!(
                "  {} ({:?}, {:?})",
                info.name, info.device_type, info.backend
            );
        }

        let present_mode = PresentMode::Mailbox;
        let format = adapter
            .get_texture_format_features(PREFERRED_FORMAT)
            .allowed_usages
            .contains(TextureUsage::RENDER_ATTACHMENT)
            .then(|| PREFERRED_FORMAT)
            .unwrap_or(adapter.get_swap_chain_preferred_format(&surface));

        debug!(
            "Using swapchain format of {:?} and present mode of {:?}",
            format, present_mode
        );

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("vg-device"),
                    features: Features::MAPPABLE_PRIMARY_BUFFERS | Features::PUSH_CONSTANTS,
                    limits: Limits {
                        max_bind_groups: 8,
                        max_storage_buffers_per_shader_stage: 5,
                        max_push_constant_size: 128,
                        ..Default::default()
                    },
                },
                None,
            )
            .await
            .expect("Failed to acquire graphics device");

        let size = window.inner_size();

        let swapchain_desc = SwapChainDescriptor {
            format,
            present_mode,
            usage: TextureUsage::RENDER_ATTACHMENT,
            width: size.width,
            height: size.height,
        };

        let swapchain = device.create_swap_chain(&surface, &swapchain_desc);

        let instance = Arc::new(instance);
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let renderer = RendererBuilder::new(RendererOptions {
            vsync: VSyncMode::Off, // we manually handle vsync
            size: size.into(),
            ambient: Default::default(),
        })
        .device(CustomDevice {
            instance: Arc::clone(&instance),
            device: Arc::clone(&device),
            queue: Arc::clone(&queue),
            info: adapter.get_info(),
        })
        .build()
        .await
        .expect("Failed to initialize rend3 renderer");

        let shaders = DefaultShaders::new(&renderer).await;
        let pipelines = DefaultPipelines::new(&renderer, &shaders).await;

        // TODO: rend3:0.0.5 crashes if there are 0 objects in the scene, create an invisible mesh
        {
            let mesh = renderer.add_mesh(Mesh {
                vertex_positions: vec![Default::default()],
                vertex_normals: vec![Default::default()],
                vertex_tangents: vec![Default::default()],
                vertex_uvs: vec![Default::default()],
                vertex_colors: vec![Default::default()],
                vertex_material_indices: vec![Default::default()],
                indices: vec![0, 0, 0],
            });

            let material = renderer.add_material(Default::default());
            let _object = renderer.add_object(Object {
                mesh,
                material,
                transform: AffineTransform {
                    transform: Default::default(),
                },
            });
        }

        #[cfg(feature = "debug")]
        let egui_pass = {
            use egui_wgpu_backend::RenderPass;
            let egui_pass = RenderPass::new(&device, format);
            egui_pass
        };

        Gfx {
            instance,
            surface,
            device,
            queue,
            window,
            swapchain_desc,
            swapchain,
            renderer,
            pipelines,
            #[cfg(feature = "debug")]
            egui_pass,
        }
    }

    fn recreate_swapchain(&mut self) {
        self.swapchain = self
            .device
            .create_swap_chain(&self.surface, &self.swapchain_desc);
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        trace!("Resizing to {}x{}", size.width, size.height);
        self.swapchain_desc.width = size.width;
        self.swapchain_desc.height = size.height;
        self.recreate_swapchain();
        self.renderer.set_options(RendererOptions {
            vsync: VSyncMode::Off, // we manually handle vsync
            size: size.into(),
            ambient: Default::default(),
        })
    }

    pub fn present(&mut self, #[cfg(feature = "debug")] debug: &mut crate::debug::DebugData) {
        self.device.poll(Maintain::Poll);

        let frame = Arc::new(self.swapchain.get_current_frame().unwrap_or_else(|e| {
            warn!("Failed to acquire swapchain frame, recreating: {}", e);
            self.recreate_swapchain();
            self.swapchain
                .get_current_frame()
                .expect("Failed to acquire new valid swapchain")
        }));

        let render_list = rend3_list::default_render_list(
            self.renderer.mode(),
            [self.swapchain_desc.width, self.swapchain_desc.height],
            &self.pipelines,
        );

        let handle = self.renderer.render(
            render_list,
            RendererOutput::ExternalSwapchain(frame.clone()),
        );

        pollster::block_on(handle);


        #[cfg(feature = "debug")]
        if debug.visible {
            let egui_start = Instant::now();
            debug.platform.begin_frame();

            let mut app_output = epi::backend::AppOutput::default();

            let mut egui_frame = epi::backend::FrameBuilder {
                info: epi::IntegrationInfo {
                    web_info: None,
                    cpu_usage: debug.last_frame_time,
                    seconds_since_midnight: None,
                    native_pixels_per_point: Some(self.window.scale_factor() as f32),
                },
                tex_allocator: &mut self.egui_pass,
                output: &mut app_output,
                repaint_signal: debug.repaint_signal.clone(),
            }
            .build();

            use epi::App;
            debug.update(&debug.platform.context(), &mut egui_frame);

            let (_out, paint_commands) = debug.platform.end_frame();
            let paint_jobs = debug.platform.context().tessellate(paint_commands);

            let frame_time = (Instant::now() - egui_start).as_secs_f32();
            debug.last_frame_time = Some(frame_time);

            let mut enc = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("egui-encoder"),
                });

            let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
                physical_width: self.swapchain_desc.width,
                physical_height: self.swapchain_desc.height,
                scale_factor: self.window.scale_factor() as f32,
            };

            self.egui_pass.update_texture(
                &self.device,
                &self.queue,
                // debug.platform.context().texture(),
                &debug.platform.context().texture(),
            );
            self.egui_pass
                .update_user_textures(&self.device, &self.queue);
            self.egui_pass.update_buffers(
                &self.device,
                &self.queue,
                &paint_jobs,
                &screen_descriptor,
            );
            self.egui_pass.execute(
                &mut enc,
                &frame.output.view,
                &paint_jobs,
                &screen_descriptor,
                None,
            );

            // Draw the debug UI
            self.queue.submit(Some(enc.finish()));
        }

    }
}
