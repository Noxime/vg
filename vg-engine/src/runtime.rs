use vg_interface::WaitReason;
use vg_runtime::executor::Instance;

use crate::Engine;

impl Engine {
    /// Run the instance until a new frame is ready
    pub(crate) fn run_frame(&mut self) {
        let Some(instance) = self.instance.get() else { return };

        loop {
            match instance.step() {
                WaitReason::Startup => continue,
                WaitReason::Present => break,
            }
        }
    }
}
