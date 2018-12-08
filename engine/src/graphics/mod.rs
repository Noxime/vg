#[cfg(feature = "backend-dx")]
extern crate gfx_backend_dx12;
#[cfg(feature = "backend-gl")]
extern crate gfx_backend_gl;
#[cfg(feature = "backend-mt")]
extern crate gfx_backend_metal;
#[cfg(feature = "backend-vk")]
extern crate gfx_backend_vulkan;
extern crate gfx_hal as hal;

use vectors::*;

mod error;
pub use self::error::GraphicsError;
mod window;
pub use self::window::*;
mod backend;
pub use self::backend::GfxBackend;
mod adapter;
pub use self::adapter::GfxAdapter;
mod device;
pub use self::device::GfxDevice;
mod swapchain;
pub use self::swapchain::GfxSwapchain;
mod renderpass;
pub use self::renderpass::GfxRenderPass; // TODO: Rename to GfxRenderpass
mod framebuffer;
pub use self::framebuffer::GfxFramebuffer;

use std::{cell::RefCell, rc::Rc};

use graphics::hal::{format::AsFormat, Device, Swapchain};
const COLOR_FORMAT: hal::format::Format = hal::format::Rgba8Srgb::SELF;
const COLOR_RANGE: hal::image::SubresourceRange =
    hal::image::SubresourceRange {
        aspects: hal::format::Aspects::COLOR,
        levels: 0..1,
        layers: 0..1,
    };

#[derive(Debug, Copy, Clone)]
#[cfg_attr(rustfmt, rustfmt_skip)]
pub enum API {
    #[cfg(feature = "backend-gl")] GL,
    #[cfg(feature = "backend-vk")] VK,
    #[cfg(feature = "backend-mt")] MT,
    #[cfg(feature = "backend-dx")] DX,
}

#[cfg_attr(rustfmt, rustfmt_skip)] 
pub enum RenderSwitch {
    #[cfg(feature = "backend-gl")] GL(RenderData<gfx_backend_gl::Backend>),
    #[cfg(feature = "backend-vk")] VK(RenderData<gfx_backend_vulkan::Backend>),
    #[cfg(feature = "backend-mt")] MT(RenderData<gfx_backend_metal::Backend>),
    #[cfg(feature = "backend-dx")] DX(RenderData<gfx_backend_dx12::Backend>),
}

pub struct RenderData<B: hal::Backend> {
    backend: GfxBackend<B>,
    device: Rc<RefCell<GfxDevice<B>>>,
    swapchain: Option<GfxSwapchain<B>>,
    renderpass: GfxRenderPass<B>,
    framebuffer: GfxFramebuffer<B>,
    viewport: hal::pso::Viewport,
    submit_queue: Vec<
        hal::command::Submit<
            B,
            hal::Graphics,
            hal::command::OneShot,
            hal::command::Primary,
        >,
    >,
}

pub struct Renderer {
    data: RenderSwitch,
}

impl Renderer {
    pub fn from(win: &mut Window) -> Self {
        trace!("creating renderer from window");

        info!("Available API paths:");
        #[cfg_attr(rustfmt, rustfmt_skip)] {
            #[cfg(feature = "backend-mt")] info!("  Metal"); 
            #[cfg(feature = "backend-dx")] info!("  DirectX 12"); 
            #[cfg(feature = "backend-vk")] info!("  Vulkan"); 
            #[cfg(feature = "backend-gl")] info!("  OpenGL"); 
        }

        #[cfg(feature = "backend-mt")]
        {
            match GfxBackend::new_mt(win) {
                Ok(b) => {
                    return Self {
                        data: RenderSwitch::MT(Self::prepare(b)),
                    }
                }
                Err(e) => debug!("Not using backend MT: {:?}", e),
            }
        }
        #[cfg(feature = "backend-dx")]
        {
            match GfxBackend::new_dx(win) {
                Ok(b) => {
                    return Self {
                        data: RenderSwitch::DX(Self::prepare(b)),
                    }
                }
                Err(e) => debug!("Not using backend DX: {:?}", e),
            }
        }
        #[cfg(feature = "backend-vk")]
        {
            match GfxBackend::new_vk(win) {
                Ok(b) => {
                    return Self {
                        data: RenderSwitch::VK(Self::prepare(b)),
                    }
                }
                Err(e) => debug!("Not using backend VK: {:?}", e),
            }
        }
        #[cfg(feature = "backend-gl")]
        {
            match GfxBackend::new_gl(win) {
                Ok(b) => {
                    return Self {
                        data: RenderSwitch::GL(Self::prepare(b)),
                    }
                }
                Err(e) => debug!("Not using backend GL: {:?}", e),
            }
        }

        error!("No backends available!");
        unimplemented!("present user with error");
    }

    fn prepare<B: hal::Backend>(mut backend: GfxBackend<B>) -> RenderData<B> {
        let device = Rc::new(RefCell::new(GfxDevice::new(
            backend.adapter.adapter.take().expect("Adapter gone"),
            &backend.surface,
        )));

        let mut swapchain =
            Some(GfxSwapchain::new(&mut backend, Rc::clone(&device)));

        let renderpass =
            GfxRenderPass::new(swapchain.as_ref().unwrap(), Rc::clone(&device));

        let framebuffer = GfxFramebuffer::new(
            &renderpass,
            swapchain.as_mut().unwrap(),
            Rc::clone(&device),
        );

        let viewport = Self::create_viewport(swapchain.as_ref().unwrap());

        RenderData {
            backend,
            device,
            swapchain,
            renderpass,
            framebuffer,
            viewport,
            submit_queue: vec![],
        }
    }

    fn create_viewport<B: hal::Backend>(
        swap: &GfxSwapchain<B>,
    ) -> hal::pso::Viewport {
        hal::pso::Viewport {
            rect: hal::pso::Rect {
                x: 0,
                y: 0,
                w: swap.extent.width as _,
                h: swap.extent.height as _,
            },
            depth: 0.0..1.0,
        }
    }

    pub fn resize(&mut self, size: Vec2<usize>) {
        debug!("Resizing to {}x{}", size.x, size.y);
        match self.data {
            #[cfg(feature = "backend-gl")]
            RenderSwitch::GL(ref mut data) => {
                data.backend.surface.get_window().resize(
                    gfx_backend_gl::glutin::dpi::LogicalSize::new(size.x as _, size.y as _)
                        .to_physical(
                            data.backend
                                .surface
                                .get_window()
                                .get_hidpi_factor(),
                        ),
                );
                Self::recreate_swapchain(data);
            }
            #[cfg(feature = "backend-mt")]
            RenderSwitch::MT(ref mut data) => Self::recreate_swapchain(data),
            #[cfg(feature = "backend-dx")]
            RenderSwitch::DX(ref mut data) => Self::recreate_swapchain(data),
            #[cfg(feature = "backend-vk")]
            RenderSwitch::VK(ref mut data) => Self::recreate_swapchain(data),
        };
    }

    fn recreate_swapchain<B: hal::Backend>(data: &mut RenderData<B>) {
        data.device
            .borrow()
            .device
            .wait_idle()
            .expect("cant wait device idle");

        // dispose old swapchain, Drop handles this
        let _ = data.swapchain.take();

        data.swapchain = Some(GfxSwapchain::new(
            &mut data.backend,
            Rc::clone(&data.device),
        ));

        data.renderpass = GfxRenderPass::new(
            data.swapchain.as_ref().unwrap(),
            Rc::clone(&data.device),
        );

        data.framebuffer = GfxFramebuffer::new(
            &data.renderpass,
            data.swapchain.as_mut().unwrap(),
            Rc::clone(&data.device),
        );

        data.viewport = Self::create_viewport(data.swapchain.as_ref().unwrap());
    }

    pub fn present(&mut self) {
        match self.data {
            #[cfg(feature = "backend-gl")]
            RenderSwitch::GL(ref mut data) => Self::draw_and_present(data),
            #[cfg(feature = "backend-mt")]
            RenderSwitch::MT(ref mut data) => Self::draw_and_present(data),
            #[cfg(feature = "backend-dx")]
            RenderSwitch::DX(ref mut data) => Self::draw_and_present(data),
            #[cfg(feature = "backend-vk")]
            RenderSwitch::VK(ref mut data) => Self::draw_and_present(data),
        }
    }

    fn draw_and_present<B: hal::Backend>(data: &mut RenderData<B>) {
        let state = {
            let sem_index = data.framebuffer.next_index();
            trace!("Presenting with frame {}", sem_index);
            let frame = {
                let (acquire_semaphore, _) =
                    data.framebuffer.get_data(None, Some(sem_index)).1.unwrap();
                data.swapchain
                    .as_mut()
                    .unwrap()
                    .swapchain
                    .as_mut()
                    .unwrap()
                    .acquire_image(
                        !0,
                        hal::FrameSync::Semaphore(acquire_semaphore),
                    )
            };
            let frame = match frame {
                Ok(i) => i,
                Err(why) => {
                    error!("Could not acquire image, skipping present and recreating swapchain: {:?}", why);
                    Self::recreate_swapchain(data);
                    return;
                }
            };

            let (fid, sid) = data
                .framebuffer
                .get_data(Some(frame as usize), Some(sem_index));
            // TODO: Unwrap == BAD!
            let (framebuffer_fence, framebuffer, command_pool) = fid.unwrap();
            let (image_acquired, image_present) = sid.unwrap();

            data.device
                .borrow()
                .device
                .wait_for_fence(framebuffer_fence, !0);
            data.device.borrow().device.reset_fence(framebuffer_fence);
            command_pool.reset();

            trace!("Submitting {} drawcalls", data.submit_queue.len());
            let submission = hal::Submission::new()
                .wait_on(&[(
                    &*image_acquired,
                    hal::pso::PipelineStage::BOTTOM_OF_PIPE,
                )])
                .signal(&[&*image_present])
                .submit(data.submit_queue.drain(..)); // TODO: Gather calls here and get rid of shite

            // submit call to device
            data.device.borrow_mut().queues.queues[0]
                .submit(submission, Some(framebuffer_fence));

            data.swapchain
                .as_ref()
                .unwrap()
                .swapchain
                .as_ref()
                .unwrap()
                .present(
                    &mut data.device.borrow_mut().queues.queues[0],
                    frame,
                    Some(&*image_present),
                )
        };
        if let Err(why) = state {
            error!("Presentation failed, recreating swapchain: {:?}", why);
            Self::recreate_swapchain(data);
        }
    }
}

impl<B: hal::Backend> Drop for RenderData<B> {
    fn drop(&mut self) {
        trace!("Dropping renderdata");
        self.device.borrow().device.wait_idle().unwrap();
        let _ = self.swapchain.take(); // drop swapchain
    }
}
