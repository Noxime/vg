use graphics::*;

pub struct GfxFramebuffer<B: hal::Backend> {
    framebuffers: Option<Vec<B::Framebuffer>>,
    framebuffer_fences: Option<Vec<B::Fence>>,
    command_pools: Option<Vec<hal::CommandPool<B, hal::Graphics>>>,
    frame_images: Option<Vec<(B::Image, B::ImageView)>>,
    acquire_semaphores: Option<Vec<B::Semaphore>>,
    present_semaphores: Option<Vec<B::Semaphore>>,
    last_ref: usize,
    device: Rc<RefCell<GfxDevice<B>>>,
}

impl<B: hal::Backend> GfxFramebuffer<B> {
    pub fn new(
        renderpass: &GfxRenderPass<B>,
        swapchain: &mut GfxSwapchain<B>,
        device: Rc<RefCell<GfxDevice<B>>>,
    ) -> Self {
        trace!("Creating framebuffers");
        let (frame_images, framebuffers) =
            match swapchain.backbuffer.take().unwrap() {
                hal::Backbuffer::Images(images) => {
                    let extent = hal::image::Extent {
                        width: swapchain.extent.width as _,
                        height: swapchain.extent.height as _,
                        depth: 1,
                    };
                    let pairs = images
                        .into_iter()
                        .map(|image| {
                            let rtv = device
                                .borrow()
                                .device
                                .create_image_view(
                                    &image,
                                    hal::image::ViewKind::D2,
                                    swapchain.format,
                                    hal::format::Swizzle::NO,
                                    COLOR_RANGE.clone(),
                                )
                                .unwrap();
                            (image, rtv)
                        })
                        .collect::<Vec<_>>();
                    let fbos = pairs
                        .iter()
                        .map(|&(_, ref rtv)| {
                            device
                                .borrow()
                                .device
                                .create_framebuffer(
                                    renderpass.renderpass.as_ref().unwrap(),
                                    Some(rtv),
                                    extent,
                                )
                                .unwrap()
                        })
                        .collect();
                    (pairs, fbos)
                }
                hal::Backbuffer::Framebuffer(fbo) => (Vec::new(), vec![fbo]),
            };

        let iter_count = if frame_images.len() != 0 {
            frame_images.len()
        } else {
            1 // GL can have zero
        };

        trace!("Framebuffer count: {}", frame_images.len());

        let mut fences: Vec<B::Fence> = vec![];
        let mut command_pools: Vec<hal::CommandPool<B, hal::Graphics>> = vec![];
        let mut acquire_semaphores: Vec<B::Semaphore> = vec![];
        let mut present_semaphores: Vec<B::Semaphore> = vec![];

        for _ in 0..iter_count {
            fences.push(device.borrow().device.create_fence(true));
            command_pools.push(
                device.borrow().device.create_command_pool_typed(
                    &device.borrow().queues,
                    hal::pool::CommandPoolCreateFlags::empty(),
                    16, // max buffers
                ),
            );

            acquire_semaphores.push(device.borrow().device.create_semaphore());
            present_semaphores.push(device.borrow().device.create_semaphore());
        }

        Self {
            frame_images: Some(frame_images),
            framebuffers: Some(framebuffers),
            framebuffer_fences: Some(fences),
            command_pools: Some(command_pools),
            present_semaphores: Some(present_semaphores),
            acquire_semaphores: Some(acquire_semaphores),
            device,
            last_ref: 0,
        }
    }

    /// get next frame index
    pub fn next_index(&mut self) -> usize {
        if self.last_ref >= self.acquire_semaphores.as_ref().unwrap().len() {
            self.last_ref = 0;
        }
        let ret = self.last_ref;
        self.last_ref += 1;
        ret
    }

    pub fn get_data(
        &mut self,
        frame_id: Option<usize>,
        sem_index: Option<usize>,
    ) -> (
        Option<(
            &mut B::Fence,
            &mut B::Framebuffer,
            &mut hal::CommandPool<B, hal::Graphics>,
        )>,
        Option<(&mut B::Semaphore, &mut B::Semaphore)>,
    ) {
        (
            if let Some(fid) = frame_id {
                Some((
                    &mut self.framebuffer_fences.as_mut().unwrap()[fid],
                    &mut self.framebuffers.as_mut().unwrap()[fid],
                    &mut self.command_pools.as_mut().unwrap()[fid],
                ))
            } else {
                None
            },
            if let Some(sid) = sem_index {
                Some((
                    &mut self.acquire_semaphores.as_mut().unwrap()[sid],
                    &mut self.present_semaphores.as_mut().unwrap()[sid],
                ))
            } else {
                None
            },
        )
    }
}

impl<B: hal::Backend> Drop for GfxFramebuffer<B> {
    fn drop(&mut self) {
        trace!("Dropping framebuffer");
        let device = &self.device.borrow().device;

        for fence in self.framebuffer_fences.take().unwrap() {
            device.wait_for_fence(&fence, !0);
            device.destroy_fence(fence);
        }

        for command_pool in self.command_pools.take().unwrap() {
            device.destroy_command_pool(command_pool.into_raw());
        }

        for acquire_semaphore in self.acquire_semaphores.take().unwrap() {
            device.destroy_semaphore(acquire_semaphore);
        }

        for present_semaphore in self.present_semaphores.take().unwrap() {
            device.destroy_semaphore(present_semaphore);
        }

        for framebuffer in self.framebuffers.take().unwrap() {
            device.destroy_framebuffer(framebuffer);
        }

        for (_, rtv) in self.frame_images.take().unwrap() {
            device.destroy_image_view(rtv);
        }
    }
}
