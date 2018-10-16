use graphics::hal::{Adapter, Backend, Limits, MemoryType, PhysicalDevice};

pub struct GfxAdapter<B: Backend> {
    adapter: Adapter<B>,
    memory_types: Vec<MemoryType>,
    limits: Limits,
}

impl<B: Backend> GfxAdapter<B> {
    pub fn new(adapters: &mut Vec<Adapter<B>>) -> Result<Self, ()> {
        if adapters.is_empty() {
            return Err(());
        }
        debug!("Available adapters:");

        for adapter in adapters.iter() {
            debug!("  {}", adapter.info.name);
        }

        Ok(Self::from_adapter(adapters.remove(0)))
    }

    fn from_adapter(adapter: Adapter<B>) -> Self {
        let memory_types =
            adapter.physical_device.memory_properties().memory_types;
        let limits = adapter.physical_device.limits();
        debug!("Using adapter: {}", adapter.info.name);

        GfxAdapter {
            adapter: adapter,
            memory_types,
            limits,
        }
    }

    pub fn info(&self) -> String {
        format!("{}", self.adapter.info.name)
    }
}
