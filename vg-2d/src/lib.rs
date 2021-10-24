use dashmap::DashMap;
use glam::{IVec2, UVec2, Vec2, Vec4};
use log::debug;
use std::{
    collections::HashMap,
    mem::{self, Discriminant},
    sync::Arc,
};
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendComponent,
    BlendFactor, BlendOperation, BlendState, Buffer, BufferAddress, BufferBinding,
    BufferBindingType, BufferDescriptor, BufferSize, BufferUsages, Color, ColorTargetState,
    ColorWrites, CommandBuffer, CommandEncoderDescriptor, CompareFunction, DepthBiasState,
    DepthStencilState, Device, DynamicOffset, Extent3d, FragmentState, LoadOp, Operations,
    PipelineLayout, PipelineLayoutDescriptor, Queue, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, ShaderStages, StencilState, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView, VertexState,
};

// mod layer;
// pub use layer::Layer;

// We can have more shapes per layer, but we have to split the draws into groups
// of MAX_SHAPES to make sure the shape data fits in the uniform buffer
const MAX_SHAPES: usize = 256;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub(crate) struct Locals {
    pub color: Vec4,
    pub props: Vec4,
    pub xyzw: Vec4,
    pub uvst: Vec4,
}

unsafe impl bytemuck::Pod for Locals {}
unsafe impl bytemuck::Zeroable for Locals {}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub(crate) struct Globals {
    pub bounds: Vec4,
}

unsafe impl bytemuck::Pod for Globals {}
unsafe impl bytemuck::Zeroable for Globals {}

pub struct RenderOutput {
    pub view: TextureView,
    pub format: TextureFormat,
}

pub struct Shape {
    kind: ShapeKind,
    color: Vec4,
    rounding: f32,
    outline: Option<f32>,
}

impl Shape {
    pub fn line(a: Vec2, b: Vec2) -> Shape {
        Shape {
            kind: ShapeKind::Line(a, b),
            ..Default::default()
        }
    }

    pub fn circle(p: Vec2) -> Shape {
        Shape {
            kind: ShapeKind::Circle(p),
            ..Default::default()
        }
    }

    pub fn rect(a: Vec2, b: Vec2, th: f32) -> Shape {
        Shape {
            kind: ShapeKind::Rect(a, b, th),
            ..Default::default()
        }
    }

    pub fn triangle(a: Vec2, b: Vec2, c: Vec2) -> Shape {
        Shape {
            kind: ShapeKind::Triangle(a, b, c),
            ..Default::default()
        }
    }
}

impl Default for Shape {
    fn default() -> Shape {
        Shape {
            kind: ShapeKind::Line(Default::default(), Default::default()),
            color: Vec4::splat(1.0),
            rounding: 0.0,
            outline: None,
        }
    }
}

pub enum ShapeKind {
    // A line from A to B
    Line(Vec2, Vec2),
    Circle(Vec2),
    Rect(Vec2, Vec2, f32),
    Triangle(Vec2, Vec2, Vec2),
}

impl Shape {
    fn as_locals(&self) -> Locals {
        match self.kind {
            ShapeKind::Line(a, b) => Locals {
                color: self.color,
                props: Vec4::new(self.rounding, self.outline.unwrap_or(0.0), 0.0, 0.0),
                xyzw: Vec4::new(a.x, a.y, b.x, b.y),
                uvst: Vec4::ZERO,
            },
            ShapeKind::Circle(p) => Locals {
                color: self.color,
                props: Vec4::new(self.rounding, self.outline.unwrap_or(0.0), 1.0, 0.0),
                xyzw: Vec4::new(p.x, p.y, 0.0, 0.0),
                uvst: Vec4::ZERO,
            },
            ShapeKind::Rect(a, b, th) => Locals {
                color: self.color,
                props: Vec4::new(self.rounding, self.outline.unwrap_or(0.0), 2.0, 0.0),
                xyzw: Vec4::new(a.x, a.y, b.x, b.y),
                uvst: Vec4::new(th, 0.0, 0.0, 0.0),
            },
            ShapeKind::Triangle(a, b, c) => Locals {
                color: self.color,
                props: Vec4::new(self.rounding, self.outline.unwrap_or(0.0), 3.0, 0.0),
                xyzw: Vec4::new(a.x, a.y, b.x, b.y),
                uvst: Vec4::new(c.x, c.y, 0.0, 0.0),
            },
        }
    }

    pub fn with_width(mut self, f: f32) -> Shape {
        self.rounding = f;
        self
    }

    pub fn with_outline(mut self, o: Option<f32>) -> Shape {
        self.outline = o;
        self
    }

    pub fn with_color(mut self, c: Vec4) -> Shape {
        self.color = c;
        self
    }
}

pub struct Renderer {
    device: Arc<Device>,
    pipeline_layout: PipelineLayout,
    pipeline: Option<(TextureFormat, RenderPipeline)>,
    module: ShaderModule,
    bind_group: BindGroup,
    locals_buffer: Buffer,
    globals_buffer: Buffer,
    depth_texture: Texture,
    depth_texture_view: TextureView,
}

impl Renderer {
    pub fn new(device: Arc<Device>, size: UVec2) -> Renderer {
        let module = device.create_shader_module(&include_wgsl!("shader.wgsl"));
        let align = device.limits().min_uniform_buffer_offset_alignment as BufferAddress;

        debug!("Shape buffer {} bytes, global buffer {} bytes", align as usize * MAX_SHAPES, mem::size_of::<Globals>());

        let locals_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("vg-2d locals buffer"),
            size: MAX_SHAPES as BufferAddress * align,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let globals_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("vg-2d globals buffer"),
            size: mem::size_of::<Globals>() as BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("vg-2d bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("vg-2d bind group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &locals_buffer,
                        offset: 0,
                        size: BufferSize::new(align as u64),
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &globals_buffer,
                        offset: 0,
                        size: BufferSize::new(mem::size_of::<Globals>() as u64),
                    }),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("vg-2d pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = None;

        let (depth_texture, depth_texture_view) = Self::create_depth(&device, size);

        Renderer {
            device,
            pipeline_layout,
            pipeline,
            module,
            bind_group,
            locals_buffer,
            globals_buffer,
            depth_texture,
            depth_texture_view,
        }
    }

    pub fn render(
        &mut self,
        queue: &Queue,
        shapes: &[Shape],
        clear: Option<Vec4>,
        output: RenderOutput,
        viewport: (Vec2, Vec2),
    ) {
        // Update per-draw properties
        let globals = Globals {
            bounds: Vec4::new(viewport.0.x, viewport.0.y, viewport.1.x, viewport.1.y),
        };
        queue.write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&globals));

        // Check that our pipeline is appropriate
        if self
            .pipeline
            .as_ref()
            .filter(|(f, _)| *f == output.format)
            .is_none()
        {
            self.pipeline = Some(self.create_pipeline(output.format));
        }
        let (_, pipeline) = self.pipeline.as_ref().unwrap();

        // Clear the screen if needed and reset depth buffer
        let mut enc = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("vg-2d clear encoder"),
            });

        {
            let mut rpass = enc.begin_render_pass(&RenderPassDescriptor {
                label: Some("vg-2d clear pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: &output.view,
                    resolve_target: None,
                    ops: Operations {
                        load: match clear {
                            Some(c) => LoadOp::Clear(Color {
                                r: c.x as f64,
                                g: c.y as f64,
                                b: c.z as f64,
                                a: c.w as f64,
                            }),
                            None => LoadOp::Load,
                        },
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1e9),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            rpass.set_pipeline(pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[0]);
            rpass.draw(0..0, 0..0);
        }

        queue.submit([enc.finish()]);

        // We can only fit so many shapes into the uniform buffer at once so work in chunks of MAX_SHAPES
        for shapes in shapes.chunks(MAX_SHAPES) {
            self.render_chunk(shapes, queue, pipeline, &output.view)
        }
    }

    fn render_chunk(
        &self,
        shapes: &[Shape],
        queue: &Queue,
        pipeline: &RenderPipeline,
        output: &TextureView,
    ) {
        let align = self.device.limits().min_uniform_buffer_offset_alignment as BufferAddress;
        // Copy current shapes into the locals buffer
        let mut data = vec![0; MAX_SHAPES * align as usize];
        for i in 0..shapes.len() {
            let offset = i * align as usize;
            data[offset..][..mem::size_of::<Locals>()]
                .copy_from_slice(bytemuck::bytes_of(&shapes[i].as_locals()))
        }
        queue.write_buffer(&self.locals_buffer, 0, bytemuck::cast_slice(&data));

        // Start drawing
        let mut enc = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("vg-2d output encoder"),
            });

        {
            let mut rpass = enc.begin_render_pass(&RenderPassDescriptor {
                label: Some("vg-2d pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: output,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            rpass.set_pipeline(&pipeline);

            for i in 0..shapes.len() {
                let offset = (i * align as usize) as DynamicOffset;

                rpass.set_bind_group(0, &self.bind_group, &[offset]);
                rpass.draw(0..6, 0..1);
            }
        }

        queue.submit([enc.finish()])
    }

    pub fn resize(&mut self, size: UVec2) {
        let (t, v) = Self::create_depth(&self.device, size);

        self.depth_texture = t;
        self.depth_texture_view = v;
    }

    fn create_pipeline(&self, format: TextureFormat) -> (TextureFormat, RenderPipeline) {
        debug!("Creating pipeline for {:?}", format);

        let pl = self
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("vg-2d output"),
                layout: Some(&self.pipeline_layout),
                vertex: VertexState {
                    module: &self.module,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                primitive: Default::default(),
                depth_stencil: Some(DepthStencilState {
                    format: TextureFormat::Depth24Plus,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::LessEqual,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: Some(FragmentState {
                    module: &self.module,
                    entry_point: "fs_main",
                    targets: &[ColorTargetState {
                        format,
                        blend: Some(BlendState {
                            color: BlendComponent {
                                src_factor: BlendFactor::SrcAlpha,
                                dst_factor: BlendFactor::OneMinusSrcAlpha,
                                operation: BlendOperation::Add,
                            },
                            alpha: BlendComponent {
                                src_factor: BlendFactor::One,
                                dst_factor: BlendFactor::One,
                                operation: BlendOperation::Add,
                            },
                        }),
                        write_mask: ColorWrites::all(),
                    }],
                }),
            });

        (format, pl)
    }

    fn create_depth(device: &Device, size: UVec2) -> (Texture, TextureView) {
        debug!("Creating depth texture for {}", size);

        let tex = device.create_texture(&TextureDescriptor {
            label: Some("vg-2d distance field depth"),
            size: Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth24Plus,
            usage: TextureUsages::RENDER_ATTACHMENT,
        });

        let view = tex.create_view(&Default::default());

        (tex, view)
    }
}

pub fn calculate_bounds(size: UVec2) -> (Vec2, Vec2) {
    let aspect = Vec2::new(size.x as f32 / size.y as f32, 1.0);
    (-Vec2::ONE * aspect, Vec2::ONE * aspect)
}
