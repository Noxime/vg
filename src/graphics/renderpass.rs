use graphics::*;

pub struct GfxRenderPass<B: hal::Backend> {
    pub renderpass: Option<B::RenderPass>,
    device: Rc<RefCell<GfxDevice<B>>>,
}

impl<B: hal::Backend> GfxRenderPass<B> {
    pub fn new(swap: &GfxSwapchain<B>, device: Rc<RefCell<GfxDevice<B>>>) -> Self {
        trace!("Creating renderpass");
        let renderpass = {
            let attachment = hal::pass::Attachment {
                format: Some(swap.format.clone()),
                samples: 1,
                ops: hal::pass::AttachmentOps::new(
                    hal::pass::AttachmentLoadOp::Clear,
                    hal::pass::AttachmentStoreOp::Store,
                ),
                stencil_ops: hal::pass::AttachmentOps::DONT_CARE,
                layouts: hal::image::Layout::Undefined
                    ..hal::image::Layout::Present,
            };

            let subpass = hal::pass::SubpassDesc {
                colors: &[(0, hal::image::Layout::ColorAttachmentOptimal)],
                depth_stencil: None,
                inputs: &[],
                resolves: &[],
                preserves: &[],
            };

            let dependency = hal::pass::SubpassDependency {
                passes: hal::pass::SubpassRef::External
                    ..hal::pass::SubpassRef::Pass(0),
                stages: hal::pso::PipelineStage::COLOR_ATTACHMENT_OUTPUT
                    ..hal::pso::PipelineStage::COLOR_ATTACHMENT_OUTPUT,
                accesses: hal::image::Access::empty()
                    ..(hal::image::Access::COLOR_ATTACHMENT_READ
                        | hal::image::Access::COLOR_ATTACHMENT_WRITE),
            };

            device.borrow().device.create_render_pass(
                &[attachment],
                &[subpass],
                &[dependency],
            )
        };

        Self {
            renderpass: Some(renderpass),
            device,
        }
    }
}

impl<B: hal::Backend> Drop for GfxRenderPass<B> {
    fn drop(&mut self) {
        trace!("Dropping renderpass");
        let device = &self.device.borrow().device;
        device.destroy_render_pass(self.renderpass.take().unwrap());
    }
}