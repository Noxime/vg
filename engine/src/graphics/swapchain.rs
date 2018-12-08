use graphics::{
    hal::{Device, Surface},
    *,
};

pub struct GfxSwapchain<B: hal::Backend> {
    pub swapchain: Option<B::Swapchain>,
    pub backbuffer: Option<hal::Backbuffer<B>>,
    device: Rc<RefCell<GfxDevice<B>>>,
    pub extent: hal::image::Extent,
    pub format: hal::format::Format,
}

impl<B: hal::Backend> GfxSwapchain<B> {
    pub fn new(
        backend: &mut GfxBackend<B>,
        device: Rc<RefCell<GfxDevice<B>>>,
    ) -> Self {
        trace!("Creating swapchain");
    
        let (caps, formats, _present_modes) =
            backend.surface.compatibility(&device.borrow().physical);

        trace!("Swapchain formats: {:?}", formats);
        let format =
            formats.map_or(hal::format::Format::Rgba8Srgb, |formats| {
                formats
                    .iter()
                    .find(|format| {
                        format.base_format().1 == hal::format::ChannelType::Srgb
                    })
                    .map(|format| *format)
                    .unwrap_or(formats[0])
            });

        debug!("Surface format: {:?}", format);
        let swap_config = hal::SwapchainConfig::from_caps(&caps, format, hal::window::Extent2D {
            width: 1280,
            height: 720,
        });
        let extent = swap_config.extent.to_extent();

        let (swapchain, backbuffer) = device.borrow().device.create_swapchain(
            &mut backend.surface,
            swap_config,
            None,
        ).unwrap();

        Self {
            swapchain: Some(swapchain),
            backbuffer: Some(backbuffer),
            device,
            extent,
            format,
        }
    }
}

impl<B: hal::Backend> Drop for GfxSwapchain<B> {
    fn drop(&mut self) {
        if let Some(swap) = self.swapchain.take() {
            trace!("Dropping swapchain");
            self.device.borrow().device.destroy_swapchain(swap);
        } else {
            warn!("Swapchain drop failure: swapchain was None");
        }
    }
}
