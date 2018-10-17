use graphics::{
    hal::{Device, Surface},
    *,
};

pub struct GfxSwapchain<B: hal::Backend> {
    swapchain: Option<B::Swapchain>,
    backbuffer: Option<hal::Backbuffer<B>>,
    device: Rc<RefCell<GfxDevice<B>>>,
    extent: hal::image::Extent,
    format: hal::format::Format,
}

impl<B: hal::Backend> GfxSwapchain<B> {
    pub fn new(
        backend: &mut GfxBackend<B>,
        device: Rc<RefCell<GfxDevice<B>>>,
        old_swapchain: Option<GfxSwapchain<B>>,
    ) -> Self {
        trace!("Creating swapchain");

        // FIXME: This line crashes sometimes when recreating, figure out why
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
        let swap_config = hal::SwapchainConfig::from_caps(&caps, format);
        let extent = swap_config.extent.to_extent();
        let (swapchain, backbuffer) = device.borrow().device.create_swapchain(
            &mut backend.surface,
            swap_config,
            old_swapchain.and_then(|s| s.swapchain),
        );

        Self {
            swapchain: Some(swapchain),
            backbuffer: Some(backbuffer),
            device,
            extent,
            format,
        }
    }
}
