//! Platform level functionality, like event loop
//! Note that desktop.rs/android.rs/etc.rs are not part of this module

use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
};

use crate::Engine;

#[cfg(target_os = "android")]
include!("android.rs");

impl Engine {
    /// Create and run an Engine with the default behavior for a winit application
    pub fn run_winit(self, event_loop: EventLoop<()>) {
        event_loop
            .run(self.event_handler())
            .expect("Event loop exited")
    }

    /// Convenience function for typical winit event receiver
    pub fn event_handler(mut self) -> impl FnMut(Event<()>, &EventLoopWindowTarget<()>) {
        move |event, target| {
            self.event(&event, target);

            if self.alive() {
                target.set_control_flow(ControlFlow::Poll)
            } else {
                target.exit();
            }
        }
    }
}
