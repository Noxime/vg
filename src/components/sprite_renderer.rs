use components::Component;
use graphics::*;
use vectors::*;

use std::mem::size_of;

#[derive(Copy, Clone)]
struct Vertex {
    pos: [f32; 2],
    tex: [f32; 2],
}

enum MyData {
    #[cfg(feature = "backend-gl")]
    GL(<GLBack as Backend>::Buffer),
    #[cfg(feature = "backend-vk")]
    VK(<VKBack as Backend>::Buffer),
    #[cfg(feature = "backend-mt")]
    MT(<MTBack as Backend>::Buffer),
    #[cfg(feature = "backend-dx")]
    DX(<DXBack as Backend>::Buffer),
}

#[cfg_attr(rustfmt, rustfmt_skip)]
const QUAD: [Vertex; 6] = [
    Vertex { pos: [0.0, 0.0], tex: [1.0, 0.0] },
    Vertex { pos: [1.0, 0.0], tex: [1.0, 0.0] },
    Vertex { pos: [0.0, 1.0], tex: [1.0, 0.0] },
    Vertex { pos: [1.0, 1.0], tex: [1.0, 0.0] },
    Vertex { pos: [0.0, 1.0], tex: [1.0, 0.0] },
    Vertex { pos: [1.0, 0.0], tex: [1.0, 0.0] },
];

pub struct SpriteRenderer {
    data: Option<MyData>,
}

impl SpriteRenderer {
    pub fn new() -> SpriteRenderer { SpriteRenderer { data: None } }
    fn _render_init<B: Backend>(
        &mut self,
        data: &mut Data<B>,
    ) -> <B as Backend>::Buffer {
        trace!("gfx init");
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

        let mut vertices = data
            .device
            .acquire_mapping_writer::<Vertex>(
                &buffer_memory,
                0..buffer_req.size,
            ).unwrap();
        vertices[0..QUAD.len()].copy_from_slice(&QUAD);
        data.device.release_mapping_writer(vertices);

        vertex_buffer
    }
    fn _render<B: Backend>(
        vbuf: &<B as Backend>::Buffer,
        data: &mut Data<B>,
    ) {
        trace!("draw");

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
            // if let Some(vertex_buffer) = self.data {
            command_buffer.bind_vertex_buffers(0, Some((vbuf, 0)));
            // }

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
    fn _render_destroy<B: Backend>(&mut self, data: &mut Data<B>) {
        if let Some(x) = self.data.take() {

        } else {
            error!("render_destroy was called but we do not have data")
        }
        trace!("gfx destroy")
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
                    MyData::GL(v) => Self::_render(v, d),
                    _ => warn!("wrong self data type"),
                },
                #[cfg(feature = "backend-vk")]
                APIData::VK(ref mut d) => match data {
                    MyData::VK(v) => Self::_render(v, d),
                    _ => warn!("wrong self data type"),
                },
                #[cfg(feature = "backend-mt")]
                APIData::MT(ref mut d) => match data {
                    MyData::MT(v) => Self::_render(v, d),
                    _ => warn!("wrong self data type"),
                },
                #[cfg(feature = "backend-dx")]
                APIData::DX(ref mut d) => match data {
                    MyData::DX(v) => Self::_render(v, d),
                    _ => warn!("wrong self data type"),
                },
            }
        }
    }

    fn render_destroy(&mut self, api_data: &mut APIData) {
        match api_data {
            #[cfg(feature = "backend-gl")]
            APIData::GL(ref mut d) => self._render_destroy(d),
            #[cfg(feature = "backend-vk")]
            APIData::VK(ref mut d) => self._render_destroy(d),
            #[cfg(feature = "backend-mt")]
            APIData::MT(ref mut d) => self._render_destroy(d),
            #[cfg(feature = "backend-dx")]
            APIData::DX(ref mut d) => self._render_destroy(d),
        }
    }
}
