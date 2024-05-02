//! Rendering and presentation related functionality
//! TODO: This is a dumb ass name

use std::sync::Arc;

use crate::prelude::*;

use wgpu::{Adapter, Device, Queue, Surface};
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowId},
};

use crate::Engine;

use self::scene::Scene;

mod canvas;
mod compositor;
mod scene;

pub struct Head {
    adapter: Arc<Adapter>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    window: Window,
    surface: Surface,
    scene: Scene,
}

impl Engine {
    /// Check if the ID matches our window. False if headless
    pub fn is_my_window(&self, id: WindowId) -> Check {
        Check::from(self.head()?.window.id() == id)
    }

    /// Create a window if the current one is closed, unless in headless mode
    pub fn ensure_window(&mut self, target: &EventLoopWindowTarget<()>) {
        if !self.config.headless && self.head.is_none() && self.between_resumes {
            self.head = match self.block_on(Head::new(target)) {
                Ok(w) => Some(w),
                Err(e) => {
                    error!("Failed to create window: {e}");
                    None
                }
            };
        }
    }

    fn head(&self) -> Check<&Head> {
        self.head.as_ref().into()
    }

    fn head_mut(&mut self) -> Check<&mut Head> {
        self.head.as_mut().into()
    }

    /// Render current frame
    pub fn render(&mut self) -> Nil {
        let head = self.head_mut()?;

        // This internally invokes 3D and 2D render
        head.render_composite();

        Nil
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) -> Nil {
        let head = self.head_mut()?;

        head.configure();

        debug!(width = size.width, height = size.height, "Resized window");
        Nil
    }

    pub fn redraw(&mut self) -> Nil {
        self.head()?.window.request_redraw();
        Nil
    }
}
