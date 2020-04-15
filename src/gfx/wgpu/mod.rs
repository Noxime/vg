use crate::{gfx::WindowMode, *};

use winit::window::{Fullscreen, Window};

// pls only use on repr(C)
unsafe fn as_bytes<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
}

pub struct Gfx {
    window: Window,
    device: wgpu::Device,
    surface: wgpu::Surface,
    swap_chain: wgpu::SwapChain,
    sc_desc: wgpu::SwapChainDescriptor,
    queue: wgpu::Queue,
    frame: Option<wgpu::SwapChainOutput>,
    pipeline: wgpu::RenderPipeline,
    // commands: Vec<wgpu::CommandBuffer>,
    bg_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    uniform_bg: wgpu::BindGroup,
    size: Option<Size>,
}

pub struct Tex {
    _tex: wgpu::Texture,
    _view: wgpu::TextureView,
    _sampler: wgpu::Sampler,
    bg: wgpu::BindGroup,
}

impl Gfx {
    pub async fn new(window: Window) -> Gfx {
        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::all(),
        )
        .await
        .expect("WGPU adapter request failed");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: wgpu::Limits::default(),
            })
            .await;

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox, // Should be configurable
        };
        let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let vs = wgpu::read_spirv(Cursor::new(&include_bytes!("vert.spv")[..])).unwrap();
        let fs = wgpu::read_spirv(Cursor::new(&include_bytes!("frag.spv")[..])).unwrap();

        let vs_mod = device.create_shader_module(&vs);
        let fs_mod = device.create_shader_module(&fs);

        let bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Uint,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false, // ???
                    },
                },
            ],
            label: None,
        });

        let uniform_buffer = device.create_buffer_mapped(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Mat>() as u64,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        uniform_buffer
            .data
            .copy_from_slice(unsafe { as_bytes(&Mat::identity()) });
        let uniform_buffer = uniform_buffer.finish();

        let uniform_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX, // 1.
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false, // 2.
                },
            }],
            label: None,
        });

        let uniform_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bgl,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &uniform_buffer,
                    // FYI: you can share a single buffer between bindings.
                    range: 0..std::mem::size_of::<Mat>() as wgpu::BufferAddress,
                },
            }],
            label: None,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bg_layout, &uniform_bgl],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_mod,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_mod,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                write_mask: wgpu::ColorWrite::ALL,
            }],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let info = adapter.get_info();
        info!(
            "Renderer: {:?} ({:?}/\"{}\")",
            info.backend, info.device_type, info.name
        );

        Gfx {
            frame: Some(swap_chain.get_next_texture().unwrap()),
            size: None,
            window,
            pipeline,
            device,
            surface,
            swap_chain,
            sc_desc,
            queue,
            bg_layout,
            uniform_bg,
            uniform_buffer,
        }
    }

    pub fn texture(&self, data: (Vec<Color>, Size)) -> Tex {
        let (data, size) = data;

        let mut buf = Vec::with_capacity(data.len() * 4);
        for p in data {
            // TODO: Float textures
            buf.push((p.r * 255.0) as u8);
            buf.push((p.g * 255.0) as u8);
            buf.push((p.b * 255.0) as u8);
            buf.push((p.a * 255.0) as u8);
        }

        let extent = wgpu::Extent3d {
            width: size.w,
            height: size.h,
            depth: 1,
        };

        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            size: extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: None,
        });

        let tex_buf = self.device.create_buffer_mapped(&wgpu::BufferDescriptor {
            label: None,
            size: buf.len() as u64,
            usage: wgpu::BufferUsage::COPY_SRC,
        });

        tex_buf.data.copy_from_slice(&buf);
        let buffer = tex_buf.finish();
        debug!(
            "Uploaded {:.2}kb of texture data",
            buf.len() as f32 / 1024.0
        );

        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        enc.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &buffer,
                offset: 0,
                bytes_per_row: 4 * size.w, // 4 elements
                rows_per_image: size.h,
            },
            wgpu::TextureCopyView {
                texture: &tex,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            extent,
        );

        self.queue.submit(&[enc.finish()]);

        let view = tex.create_default_view();

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Always,
        });

        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bg_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        Tex {
            _tex: tex,
            _view: view,
            _sampler: sampler,
            bg,
        }
    }

    pub fn fill(&self, color: Color) {
        let frame = match &self.frame {
            Some(frame) => frame,
            None => {
                warn!("Can't render without an active swapchain image");
                return;
            }
        };
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: color.r as f64,
                        g: color.g as f64,
                        b: color.b as f64,
                        a: color.a as f64,
                    },
                }],
                depth_stencil_attachment: None,
            });
        }

        self.queue.submit(&[encoder.finish()]);
    }

    pub fn draw(&self, texture: &gfx::Texture, instances: &[Mat]) {
        let size = self.window.inner_size();

        let aspect = match size.height as f32 / size.width as f32 {
            x if x < 1.0 => vek::Vec3::new(x, 1.0, 1.0),
            y => vek::Vec3::new(1.0, 1.0 / y, 1.0),
        };
        let view = Mat::identity().scaled_3d(aspect);

        let frame = match &self.frame {
            Some(frame) => frame,
            None => {
                warn!("Can't render without an active swapchain image");
                return;
            }
        };
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        for mat in instances {
            let staging_buf = self.device.create_buffer_mapped(&wgpu::BufferDescriptor {
                label: None,
                size: std::mem::size_of::<Mat>() as u64,
                usage: wgpu::BufferUsage::COPY_SRC,
            });

            staging_buf
                .data
                .copy_from_slice(unsafe { as_bytes(&(view * *mat)) });
            let staging_buf = staging_buf.finish();

            encoder.copy_buffer_to_buffer(
                &staging_buf,
                0,
                &self.uniform_buffer,
                0,
                std::mem::size_of::<Mat>() as _,
            );

            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Load,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color::TRANSPARENT,
                    }],
                    depth_stencil_attachment: None,
                });
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &texture.tex.bg, &[]);
                pass.set_bind_group(1, &self.uniform_bg, &[]);

                pass.draw(0..6, 0..1); // draw 2 tris
            }
        }

        self.queue.submit(&[encoder.finish()]);
    }

    pub fn handle(&mut self, event: &Event) {
        match event {
            Event::Resize(size) => self.size = Some(*size),
            Event::Exit => drop(self.frame.take()),
            _ => (),
        }
    }

    pub fn present(&mut self) {
        // presents last frame
        drop(self.frame.take());

        if let Some(size) = self.size.take() {
            self.sc_desc.width = size.w;
            self.sc_desc.height = size.h;
            self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
            debug!("Recreated swapchain at {}", size);
        }

        self.frame = Some(
            self.swap_chain
                .get_next_texture()
                .expect("Could not get new swapchain target"),
        );
    }
}

impl Vg {
    pub fn title(&mut self, title: impl AsRef<str>) {
        self.gfx.window.set_title(title.as_ref());
    }
    pub fn resize(&mut self, mode: WindowMode) {
        let monitor = self.gfx.window.current_monitor();
        match mode {
            WindowMode::Fullscreen => {
                let mut vm = None;
                for v in monitor.video_modes() {
                    vm = Some(match vm {
                        Some(vm) => {
                            if v > vm {
                                v
                            } else {
                                vm
                            }
                        }
                        None => v,
                    });
                }
                self.gfx.window.set_fullscreen(Some(Fullscreen::Exclusive(
                    vm.expect("Fullscreen not supported"),
                )));
            }
            WindowMode::Borderless => {
                self.gfx
                    .window
                    .set_fullscreen(Some(Fullscreen::Borderless(monitor)));
            }
            WindowMode::Window => {
                self.gfx.window.set_fullscreen(None);
            }
        }
    }
}
