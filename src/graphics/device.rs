use graphics::*;
use graphics::hal::Surface;

pub struct GfxDevice<B: hal::Backend> {
    pub device: B::Device,
    pub physical: B::PhysicalDevice,
    pub queues: hal::QueueGroup<B, hal::Graphics>,
}

impl<B: hal::Backend> GfxDevice<B> {
    pub fn new(mut adapter: hal::Adapter<B>, surface: &B::Surface) -> Self {
        let (device, queues) = adapter
            .open_with::<_, hal::Graphics>(1, |fam| {
                surface.supports_queue_family(fam)
            })
            .expect("No graphics family");
        trace!("Created a logical device");
        Self {
            device,
            physical: adapter.physical_device,
            queues,
        }
    }
}
