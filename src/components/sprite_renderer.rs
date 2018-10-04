extern crate image as image_crate;

use components::Component;
use graphics::*;
use std::io::{Cursor, Read};
use vectors::*;

use std::mem::size_of;

enum MyData {
    #[cfg(feature = "backend-gl")]
    GL(
        (
            <GLBack as Backend>::Buffer,
            <GLBack as Backend>::Memory,
            <GLBack as Backend>::DescriptorSet,
        ),
    ),
    #[cfg(feature = "backend-vk")]
    VK(
        (
            <VKBack as Backend>::Buffer,
            <VKBack as Backend>::Memory,
            <VKBack as Backend>::DescriptorSet,
        ),
    ),
    #[cfg(feature = "backend-mt")]
    MT(
        (
            <MTBack as Backend>::Buffer,
            <MTBack as Backend>::Memory,
            <MTBack as Backend>::DescriptorSet,
        ),
    ),
    #[cfg(feature = "backend-dx")]
    DX(
        (
            <DXBack as Backend>::Buffer,
            <DXBack as Backend>::Memory,
            <DXBack as Backend>::DescriptorSet,
        ),
    ),
}

#[cfg_attr(rustfmt, rustfmt_skip)]
const QUAD: [Vertex; 6] = [
    Vertex { pos: [0.0, 0.0], tex: [0.0, 0.0] },
    Vertex { pos: [1.0, 0.0], tex: [1.0, 0.0] },
    Vertex { pos: [0.0, 1.0], tex: [0.0, 1.0] },
    Vertex { pos: [1.0, 1.0], tex: [1.0, 1.0] },
    Vertex { pos: [0.0, 1.0], tex: [0.0, 1.0] },
    Vertex { pos: [1.0, 0.0], tex: [1.0, 0.0] },
];

pub struct SpriteRenderer {
    image: image_crate::RgbaImage,
    data: Option<MyData>,
}

impl SpriteRenderer {
    pub fn new(image_data: &[u8]) -> SpriteRenderer {
        SpriteRenderer {
            image: image_crate::load(
                Cursor::new(&image_data[..]),
                image_crate::PNG,
            ).unwrap()
            .to_rgba(),
            data: None,
        }
    }
    fn _render_init<B: Backend>(
        &mut self,
        data: &mut Data<B>,
    ) -> (
        <B as Backend>::Buffer,
        <B as Backend>::Memory,
        <B as Backend>::DescriptorSet,
    ) {
        trace!("gfx init");

        // vertex buffer
        let stride = size_of::<Vertex>() as u64;
        let size = QUAD.len() as u64 * stride;

        let buffer_unbound = data
            .device
            .create_buffer(size, buffer::Usage::VERTEX)
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
        {
            let mut vertices = data
                .device
                .acquire_mapping_writer::<Vertex>(
                    &buffer_memory,
                    0..buffer_req.size,
                ).unwrap();
            vertices[0..QUAD.len()].copy_from_slice(&QUAD);
            data.device.release_mapping_writer(vertices);
        }

        // TODO: Texture upload

        let (width, height) = self.image.dimensions();
        let kind =
            image::Kind::D2(width as image::Size, height as image::Size, 1, 1);
        let row_alignment_mask =
            data.adapter
                .physical_device
                .limits()
                .min_buffer_copy_pitch_alignment as u32
                - 1;
        let image_stride = 4usize;
        let row_pitch = (width * image_stride as u32 + row_alignment_mask)
            & !row_alignment_mask;
        let upload_size = (height * row_pitch) as u64;

        let image_buffer_unbound = data
            .device
            .create_buffer(upload_size, buffer::Usage::TRANSFER_SRC)
            .unwrap();
        let image_mem_reqs =
            data.device.get_buffer_requirements(&image_buffer_unbound);
        let image_upload_memory = data
            .device
            .allocate_memory(upload_type, image_mem_reqs.size)
            .unwrap();
        let image_upload_buffer = data
            .device
            .bind_buffer_memory(&image_upload_memory, 0, image_buffer_unbound)
            .unwrap();

        // copy image data into staging buffer
        {
            let mut _data = data
                .device
                .acquire_mapping_writer::<u8>(
                    &image_upload_memory,
                    0..image_mem_reqs.size,
                ).unwrap();
            for y in 0..height as usize {
                let row = &(*self.image)[y * (width as usize) * image_stride
                                             ..(y + 1)
                                                 * (width as usize)
                                                 * image_stride];
                let dest_base = y * row_pitch as usize;
                _data[dest_base..dest_base + row.len()].copy_from_slice(row);
            }
            data.device.release_mapping_writer(_data);
        }

        let image_unbound = data
            .device
            .create_image(
                kind,
                1,
                data.format,
                image::Tiling::Optimal,
                image::Usage::TRANSFER_DST | image::Usage::SAMPLED,
                image::ViewCapabilities::empty(),
            ).unwrap(); // TODO: usage
        let image_req = data.device.get_image_requirements(&image_unbound);

        let device_type = data
            .adapter
            .physical_device
            .memory_properties()
            .memory_types
            .iter()
            .enumerate()
            .position(|(id, memory_type)| {
                image_req.type_mask & (1 << id) != 0 && memory_type
                    .properties
                    .contains(memory::Properties::DEVICE_LOCAL)
            }).unwrap()
            .into();

        let image_memory = data
            .device
            .allocate_memory(device_type, image_req.size)
            .unwrap();

        let image_texture = data
            .device
            .bind_image_memory(&image_memory, 0, image_unbound)
            .unwrap();
        let color_range = SubresourceRange {
            aspects: Aspects::COLOR,
            levels: 0..1,
            layers: 0..1,
        };
        let image_srv = data
            .device
            .create_image_view(
                &image_texture,
                image::ViewKind::D2,
                data.format,
                Swizzle::NO,
                color_range.clone(),
            ).unwrap();

        let sampler = data.device.create_sampler(image::SamplerInfo::new(
            image::Filter::Nearest,
            image::WrapMode::Tile,
        ));

        let mut desc_pool = data.device.create_descriptor_pool(
            1, // sets
            &[
                pso::DescriptorRangeDesc {
                    ty: pso::DescriptorType::SampledImage,
                    count: 1,
                },
                pso::DescriptorRangeDesc {
                    ty: pso::DescriptorType::Sampler,
                    count: 1,
                },
            ],
        );

        let set_layout = data.device.create_descriptor_set_layout(
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
            ],
            &[],
        );

        let desc_set = desc_pool.allocate_set(&set_layout).unwrap();

        data.device.write_descriptor_sets(vec![
            pso::DescriptorSetWrite {
                set: &desc_set,
                binding: 0,
                array_offset: 0,
                descriptors: Some(pso::Descriptor::Image(
                    &image_srv,
                    image::Layout::Undefined,
                )),
            },
            pso::DescriptorSetWrite {
                set: &desc_set,
                binding: 1,
                array_offset: 0,
                descriptors: Some(pso::Descriptor::Sampler(&sampler)),
            },
        ]);

        // copy buffer to texture
        {
            let submit = {
                let mut cmd_buffer =
                    data.command_pool.acquire_command_buffer(false);

                let image_barrier = memory::Barrier::Image {
                    states: (image::Access::empty(), image::Layout::Undefined)
                        ..(
                            image::Access::TRANSFER_WRITE,
                            image::Layout::TransferDstOptimal,
                        ),
                    target: &image_texture,
                    range: color_range.clone(),
                };

                cmd_buffer.pipeline_barrier(
                    PipelineStage::TOP_OF_PIPE..PipelineStage::TRANSFER,
                    memory::Dependencies::empty(),
                    &[image_barrier],
                );

                cmd_buffer.copy_buffer_to_image(
                    &image_upload_buffer,
                    &image_texture,
                    image::Layout::TransferDstOptimal,
                    &[command::BufferImageCopy {
                        buffer_offset: 0,
                        buffer_width: row_pitch / (image_stride as u32),
                        buffer_height: height as u32,
                        image_layers: image::SubresourceLayers {
                            aspects: Aspects::COLOR,
                            level: 0,
                            layers: 0..1,
                        },
                        image_offset: image::Offset { x: 0, y: 0, z: 0 },
                        image_extent: image::Extent {
                            width,
                            height,
                            depth: 1,
                        },
                    }],
                );

                let image_barrier = memory::Barrier::Image {
                    states: (
                        image::Access::TRANSFER_WRITE,
                        image::Layout::TransferDstOptimal,
                    )
                        ..(
                            image::Access::SHADER_READ,
                            image::Layout::ShaderReadOnlyOptimal,
                        ),
                    target: &image_texture,
                    range: color_range.clone(),
                };
                cmd_buffer.pipeline_barrier(
                    PipelineStage::TRANSFER..PipelineStage::FRAGMENT_SHADER,
                    memory::Dependencies::empty(),
                    &[image_barrier],
                );

                cmd_buffer.finish()
            };

            let submission = Submission::new().submit(Some(submit));
            data.queue_group.queues[0]
                .submit(submission, Some(&mut data.frame_fence));

            data.device.wait_for_fence(&data.frame_fence, !0);
        }

        debug!("Init done");
        (vertex_buffer, buffer_memory, desc_set)
    }

    fn _render<B: Backend>(
        vbuf: &<B as Backend>::Buffer,
        desc_set: &<B as Backend>::DescriptorSet,
        data: &mut Data<B>,
    ) {
        // trace!("draw");

        data.command_buffers.push({
            let mut command_buffer =
                data.command_pool.acquire_command_buffer(false);
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
            // if let Some(vertex_buffer) = self.data {
            command_buffer.bind_vertex_buffers(0, Some((vbuf, 0)));
            command_buffer.bind_graphics_descriptor_sets(
                &data.pipeline_layout,
                0,
                Some(desc_set),
                &[],
            );

            {
                let mut encoder = command_buffer.begin_render_pass_inline(
                    &data.render_pass,
                    &data.framebuffers[data.frame_index as usize],
                    viewport.rect,
                    &[ClearValue::Color(ClearColor::Float([
                        1.0, 0.0, 1.0, 1.0,
                    ]))],
                );
                encoder.draw(0..(QUAD.len() as u32), 0..1);
            }

            command_buffer.finish()
        });
    }
}

impl Component for SpriteRenderer {
    fn render_init(&mut self, api_data: &mut APIData) {
        self.data = Some(match api_data {
            #[cfg(feature = "backend-gl")]
            APIData::GL(ref mut d) => MyData::GL(self._render_init(d)),
            #[cfg(feature = "backend-vk")]
            APIData::VK(ref mut d) => MyData::VK(self._render_init(d)),
            #[cfg(feature = "backend-mt")]
            APIData::MT(ref mut d) => MyData::MT(self._render_init(d)),
            #[cfg(feature = "backend-dx")]
            APIData::DX(ref mut d) => MyData::DX(self._render_init(d)),
        });
    }

    fn render(&mut self, api_data: &mut APIData) {
        if let Some(ref data) = self.data {
            match api_data {
                #[cfg(feature = "backend-gl")]
                APIData::GL(ref mut d) => match data {
                    MyData::GL(v) => Self::_render(&v.0, &v.2, d),
                    _ => warn!("wrong self data type"),
                },
                #[cfg(feature = "backend-vk")]
                APIData::VK(ref mut d) => match data {
                    MyData::VK(v) => Self::_render(&v.0, &v.2, d),
                    _ => warn!("wrong self data type"),
                },
                #[cfg(feature = "backend-mt")]
                APIData::MT(ref mut d) => match data {
                    MyData::MT(v) => Self::_render(&v.0, &v.2, d),
                    _ => warn!("wrong self data type"),
                },
                #[cfg(feature = "backend-dx")]
                APIData::DX(ref mut d) => match data {
                    MyData::DX(v) => Self::_render(&v.0, &v.2, d),
                    _ => warn!("wrong self data type"),
                },
            }
        }
    }

    fn render_destroy(&mut self, api_data: &mut APIData) {
        if let Some(x) = self.data.take() {
            match api_data {
                #[cfg(feature = "backend-gl")]
                APIData::GL(ref mut d) => match x {
                    MyData::GL((buf, mem, desc_set)) => {
                        d.device.destroy_buffer(buf);
                        d.device.free_memory(mem);
                    }
                    _ => warn!("wrong self data type"),
                },
                #[cfg(feature = "backend-vk")]
                APIData::VK(ref mut d) => match x {
                    MyData::VK((buf, mem, desc_set)) => {
                        d.device.destroy_buffer(buf);
                        d.device.free_memory(mem);
                    }
                    _ => warn!("wrong self data type"),
                },
                #[cfg(feature = "backend-mt")]
                APIData::MT(ref mut d) => match x {
                    MyData::MT((buf, mem, desc_set)) => {
                        d.device.destroy_buffer(buf);
                        d.device.free_memory(mem);
                    }
                    _ => warn!("wrong self data type"),
                },
                #[cfg(feature = "backend-dx")]
                APIData::DX(ref mut d) => match x {
                    MyData::DX((buf, mem, desc_set)) => {
                        d.device.destroy_buffer(buf);
                        d.device.free_memory(mem);
                    }
                    _ => warn!("wrong self data type"),
                },
            }
        }
    }
}
