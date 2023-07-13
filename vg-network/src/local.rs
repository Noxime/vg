use vg_interface::WaitReason;
use vg_runtime::executor::Instance;

pub struct Local<I: Instance> {
    instance: I,
}

impl<I: Instance> Local<I> {
    pub fn new(instance: I) -> Self {
        Self { instance }
    }

    /// Lock events and execute tick
    pub fn tick(&mut self) {
        // Execute until tick is complete
        loop {
            match self.instance.step() {
                WaitReason::Startup => (),
                WaitReason::Present => break,
            }
        }
    }
}
