use wgpu::util::*;
use wgpu::*;

use crate::{assets::Asset, Transform};

use serde::{Deserialize, Serialize};
use ultraviolet::Vec4;

mod mesh;
pub use mesh::{Mesh, Vertex};
mod graph;
pub use graph::Graph;
mod uniforms;
use uniforms::Uniforms;
mod camera;
pub use camera::Camera;
mod light;
pub use light::Light;
mod texture;
pub use texture::{Env, Texture};
pub mod material;
pub use material::Material;
pub mod ui;

#[derive(Serialize, Deserialize)]
pub struct Model {
    asset: Asset,
    trans: Transform,
}

impl From<Asset> for Model {
    fn from(asset: Asset) -> Model {
        Model {
            asset,
            trans: Transform::identity() 
        }
    }
}

impl<P: AsRef<std::path::Path>> From<P> for Model {
    fn from(path: P) -> Model {
        Model { 
            asset: path.into(), 
            trans: Transform::identity()
        }
    }
}

// linear color backbuffer
const FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;

pub struct Gfx {
    pub(crate) window: winit::window::Window,
    pub(crate) instance: Instance,
    pub(crate) adapter: Adapter,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) swapchain: SwapChain,
    pub(crate) surface: Surface,
    pub(crate) pipelines: [RenderPipeline; std::mem::variant_count::<ShadingModel>()],
    pub(crate) uniforms: Uniforms,
    pub(crate) uniform_buffer: Buffer,
    pub(crate) uniform_bind_group: BindGroup,
    pub(crate) texture_bind_group_layout: BindGroupLayout,
    pub(crate) env_bind_group_layout: BindGroupLayout,
    pub(crate) default_env: Option<Env>,
    pub(crate) default_texture: Option<[Texture; 8]>,
    pub(crate) render_texture: Texture,
    pub(crate) depth_texture: Texture,
    pub(crate) fullscreen_quad: Buffer,
    pub(crate) render_scale: f32,
    pub(crate) egui_pass: egui_wgpu_backend::RenderPass,
    pub(crate) egui_platform: egui_winit_platform::Platform,
    pub(crate) egui_time: std::time::Instant,
}

enum ShadingModel {
    Lit = 0,
    Unlit,
    Post,
}

impl Gfx {
    pub async fn new(ev: &winit::event_loop::EventLoop<()>) -> Self {
        let window = winit::window::WindowBuilder::new().build(ev).unwrap();
        let instance = Instance::new(BackendBit::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: Default::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&Default::default(), None)
            .await
            .unwrap();

        let swapchain = device.create_swap_chain(&surface, &create_swapchain_desc(&window));

        window.set_title(&format!("VG ({:?})", adapter.get_info().backend));

        let uniforms = Uniforms::new();
        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("uniforms"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform bind group layout"),
            });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(uniform_buffer.slice(..)),
            }],
            label: Some("uniform bind group"),
        });

        let bind_entries: Vec<_> = (0..7)
            .map(|i| {
                vec![
                    BindGroupLayoutEntry {
                        binding: i * 2,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::SampledTexture {
                            multisampled: false,
                            dimension: TextureViewDimension::D2,
                            component_type: TextureComponentType::Uint, // TODO: HDR textures
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: i * 2 + 1,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::Sampler { comparison: false },
                        count: None,
                    },
                ]
            })
            .flatten()
            .collect();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: bind_entries.as_slice(),
                label: Some("texture bind group layout"),
            });

        let env_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("env bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        dimension: TextureViewDimension::Cube,
                        component_type: TextureComponentType::Float,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler { comparison: false },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        dimension: TextureViewDimension::Cube,
                        component_type: TextureComponentType::Float,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("render pipeline layout"),
            bind_group_layouts: &[
                &uniform_bind_group_layout,
                &texture_bind_group_layout,
                &env_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let create_pipeline = |name, vs, fs, post| {
            let vs_mod = device.create_shader_module(vs);
            let fs_mod = device.create_shader_module(fs);

            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(name),
                layout: Some(&pipeline_layout),
                vertex_stage: ProgrammableStageDescriptor {
                    module: &vs_mod,
                    entry_point: "main",
                },
                fragment_stage: Some(ProgrammableStageDescriptor {
                    module: &fs_mod,
                    entry_point: "main",
                }),
                rasterization_state: Some(RasterizationStateDescriptor {
                    front_face: FrontFace::Ccw,
                    cull_mode: CullMode::Back,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                    clamp_depth: false,
                }),
                color_states: &[ColorStateDescriptor {
                    format: if post {
                        FORMAT
                    } else {
                        TextureFormat::Rgba16Float
                    },
                    color_blend: BlendDescriptor {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                        operation: BlendOperation::Add,
                    },
                    alpha_blend: BlendDescriptor {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::DstAlpha,
                        operation: BlendOperation::Add,
                    },
                    write_mask: ColorWrite::ALL,
                }],
                primitive_topology: PrimitiveTopology::TriangleList,
                depth_stencil_state: if !post {
                    Some(DepthStencilStateDescriptor {
                        format: TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: CompareFunction::Less,
                        stencil: Default::default(),
                    })
                } else {
                    None
                },
                vertex_state: VertexStateDescriptor {
                    index_format: IndexFormat::Uint32,
                    vertex_buffers: &[mesh::Vertex::desc()],
                },
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            })
        };

        let pipelines = [
            create_pipeline(
                "lit",
                include_spirv!(concat!(env!("OUT_DIR"), "/surface-default.vs.spv")),
                include_spirv!(concat!(env!("OUT_DIR"), "/surface-pbr.fs.spv")),
                false,
            ),
            create_pipeline(
                "unlit",
                include_spirv!(concat!(env!("OUT_DIR"), "/surface-default.vs.spv")),
                include_spirv!(concat!(env!("OUT_DIR"), "/surface-unlit.fs.spv")),
                false,
            ),
            create_pipeline(
                "post",
                include_spirv!(concat!(env!("OUT_DIR"), "/post-uber.vs.spv")),
                include_spirv!(concat!(env!("OUT_DIR"), "/post-uber.fs.spv")),
                true,
            ),
        ];

        let size: (u32, u32) = window.inner_size().into();

        let render_scale = 1.0;
        let fb_size = (
            (size.0 as f32 * render_scale) as u32,
            (size.1 as f32 * render_scale) as u32,
        );

        let depth_texture = Texture::new_depth(&device, fb_size);
        let render_texture = Texture::new_render(&device, fb_size, FORMAT);

        let egui_platform =
            egui_winit_platform::Platform::new(egui_winit_platform::PlatformDescriptor {
                physical_width: size.0,
                physical_height: size.1,
                scale_factor: window.scale_factor(),
                font_definitions: egui::FontDefinitions::with_pixels_per_point(
                    window.scale_factor() as f32,
                ),
                style: Default::default(),
            });

        let egui_pass = egui_wgpu_backend::RenderPass::new(&device, FORMAT);

        let fs_vertices = vec![
            Vertex::new(1.0, 1.0, 0.0).with_uv((1.0, 0.0).into()),
            Vertex::new(-1.0, 1.0, 0.0).with_uv((0.0, 0.0).into()),
            Vertex::new(1.0, -1.0, 0.0).with_uv((1.0, 1.0).into()),
            Vertex::new(-1.0, 1.0, 0.0).with_uv((0.0, 0.0).into()),
            Vertex::new(-1.0, -1.0, 0.0).with_uv((0.0, 1.0).into()),
            Vertex::new(1.0, -1.0, 0.0).with_uv((1.0, 1.0).into()),
        ];

        let fullscreen_quad = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("fullscreen quad"),
            contents: bytemuck::cast_slice(&fs_vertices),
            usage: BufferUsage::VERTEX,
        });

        // let shadow_texture = [
        //     Texture::new_depth(&device, (512, 512)),
        //     Texture::new_depth(&device, (512, 512)),
        // ];

        Gfx {
            window,
            instance,
            adapter,
            device,
            queue,
            swapchain,
            surface,
            pipelines,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            texture_bind_group_layout,
            env_bind_group_layout,
            default_env: None,
            default_texture: None,
            render_texture,
            depth_texture,
            render_scale,
            egui_pass,
            egui_platform,
            fullscreen_quad,
            egui_time: std::time::Instant::now(),
        }
    }

    fn update_uniforms(&self, enc: &mut CommandEncoder) {
        let data = &[self.uniforms];
        let data = bytemuck::cast_slice(data);
        let staging = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("uniform staging buffer"),
            contents: data,
            usage: BufferUsage::COPY_SRC,
        });

        enc.copy_buffer_to_buffer(
            &staging,
            0,
            &self.uniform_buffer,
            0,
            data.len() as BufferAddress,
        );
    }

    pub fn set_render_scale(&mut self, scale: f32) {
        self.render_scale = scale;
    }

    pub async fn present<'a>(
        &mut self,
        ui: &mut impl egui::app::App,
        graph: Graph<'a>,
        camera: Camera,
    ) {
        // lazy init default textures, should be moved elsewhere really
        if self.default_texture.is_none() {
            let b = bytemuck::bytes_of(&[0xFFu8; 4]);
            let s = texture::TextureSettings {
                channels: 4,
                hdr: false,
                linear: true,
            };
            self.default_texture = Some([
                Texture::new(self, (1, 1), b, s),
                Texture::new(self, (1, 1), b, s),
                Texture::new(self, (1, 1), b, s),
                Texture::new(self, (1, 1), b, s),
                Texture::new(self, (1, 1), b, s),
                Texture::new(self, (1, 1), b, s),
                Texture::new(self, (1, 1), b, s),
                Texture::new(self, (1, 1), b, s),
            ]);
        }

        if self.default_env.is_none() {
            let b = bytemuck::bytes_of(&[0.025f32; 4 * 32 * 32]);
            self.default_env = Some(Env::new(self, (1, 1), [[b; 2]; 6]));
        }

        let env = graph.env.unwrap_or(self.default_env.as_ref().unwrap());

        let frame = match self.swapchain.get_current_frame() {
            Ok(frame) => frame,
            Err(_e) => {
                let desc = create_swapchain_desc(&self.window);

                let fb_size = (
                    (desc.width as f32 * self.render_scale) as u32,
                    (desc.height as f32 * self.render_scale) as u32,
                );

                self.render_texture = Texture::new_render(&self.device, fb_size, FORMAT);
                self.depth_texture = Texture::new_depth(&self.device, fb_size);

                self.swapchain = self.device.create_swap_chain(&self.surface, &desc);
                self.swapchain.get_current_frame().unwrap()
            }
        };

        let size = self.window.inner_size().into();

        self.uniforms.update(size, camera);
        for (i, light) in graph.lights.iter().enumerate() {
            self.uniforms.lights[i] = **light;
        }
        self.uniforms.light_count = graph.lights.len() as u32;

        let mut enc = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("present encoder"),
            });

        {
            self.update_uniforms(&mut enc);
            let mut bg_vec = vec![];
            let mut pass = enc.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &self.render_texture.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.0,
                            g: 0.15,
                            b: 0.3,
                            a: 0.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            pass.set_pipeline(&self.pipelines[ShadingModel::Lit as usize]);
            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            pass.set_bind_group(2, &env.bind_group, &[]);

            for (_, m) in graph.meshes.iter() {
                bg_vec.push(get_bind_group(self, m));
            }

            for ((mesh, _material), bg) in graph.meshes.iter().zip(bg_vec.iter()) {
                pass.set_bind_group(1, bg, &[]);

                pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                pass.set_index_buffer(mesh.index_buffer.slice(..));
                pass.draw_indexed(0..mesh.count, 0, 0..1);
                // pass.draw(0..3, 0..1);
            }
        }

        // finish rendering the main frame
        // Do post process, blit framebuffer
        {
            // self.uniforms.view = ultraviolet::Mat4::identity();
            // self.uniforms.projection = ultraviolet::Mat4::identity();
            // self.update_uniforms(&mut enc);
            let blit_material = Material {
                color: material::Source::Texture(&self.render_texture),
                ..Material::default()
            };

            let bg = get_bind_group(self, &blit_material);
            let mut pass = enc.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &frame.output.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 1.0,
                            g: 0.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            pass.set_pipeline(&self.pipelines[ShadingModel::Post as usize]);

            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            pass.set_bind_group(1, &bg, &[]);
            pass.set_bind_group(2, &env.bind_group, &[]);

            pass.set_vertex_buffer(0, self.fullscreen_quad.slice(..));
            pass.draw(0..6, 0..1);
        }

        // Draw UI
        {
            self.egui_platform.update_time(self.egui_time.elapsed().as_secs_f64());

            self.egui_platform.begin_frame();
            let mut integration_context = egui::app::IntegrationContext {
                info: egui::app::IntegrationInfo {
                    web_info: None,
                    cpu_usage: None,
                    seconds_since_midnight: None,
                    native_pixels_per_point: Some(self.window.scale_factor() as f32),
                },
                tex_allocator: Some(&mut self.egui_pass),
                output: Default::default(),
            };

            ui.ui(&self.egui_platform.context(), &mut integration_context);
            // self.egui_time = std::time::Instant::now();

            let (_output, paint_commands) = self.egui_platform.end_frame();
            let paint_jobs = self.egui_platform.context().tesselate(paint_commands);
            let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
                physical_width: size.0,
                physical_height: size.1,
                scale_factor: self.window.scale_factor() as f32,
            };
            self.egui_pass.update_texture(
                &self.device,
                &self.queue,
                &self.egui_platform.context().texture(),
            );
            self.egui_pass
                .update_user_textures(&self.device, &self.queue);
            self.egui_pass.update_buffers(
                &mut self.device,
                &mut self.queue,
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
        }

        self.queue.submit(Some(enc.finish()));
        drop(frame);
        self.device.poll(Maintain::Wait);
    }
}

fn create_swapchain_desc(w: &winit::window::Window) -> SwapChainDescriptor {
    let size = w.inner_size();
    SwapChainDescriptor {
        usage: TextureUsage::OUTPUT_ATTACHMENT,
        format: FORMAT,
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Mailbox,
    }
}

// Get the texture out of a source, or get/generate a new const texture
fn get_tex<'a, T: material::AsBytes>(
    gfx: &Gfx,
    default: &'a Texture,
    source: &'a material::Source<'a, T>,
) -> &'a Texture {
    match source {
        material::Source::Texture(tex) => &tex,
        material::Source::Value(value) => {
            default.update(gfx, &value.bytes());
            default
        }
    }
}

fn get_bind_group<'a>(gfx: &'a Gfx, m: &'a Material) -> BindGroup {
    let d = gfx.default_texture.as_ref().unwrap();
    let color = get_tex(gfx, &d[0], &m.color);
    let reflectance = get_tex(gfx, &d[1], &m.reflectance);
    let roughness = get_tex(gfx, &d[2], &m.roughness);
    let metallic = get_tex(gfx, &d[3], &m.metallic);
    let clear_coat = get_tex(gfx, &d[4], &m.clear_coat);
    let clear_coat_roughness = get_tex(gfx, &d[5], &m.clear_coat_roughness);
    let emission = get_tex(gfx, &d[6], &m.emission);
    // TODO: Normals

    let mut entries = vec![];
    for (i, t) in [
        color,
        reflectance,
        roughness,
        metallic,
        clear_coat,
        clear_coat_roughness,
        emission,
    ]
    .iter()
    .enumerate()
    {
        entries.push(BindGroupEntry {
            binding: i as u32 * 2,
            resource: BindingResource::TextureView(&t.view),
        });
        entries.push(BindGroupEntry {
            binding: i as u32 * 2 + 1,
            resource: BindingResource::Sampler(&t.sampler),
        });
    }

    gfx.device.create_bind_group(&BindGroupDescriptor {
        label: Some("bind group"),
        layout: &gfx.texture_bind_group_layout,
        entries: entries.as_slice(),
    })
}
