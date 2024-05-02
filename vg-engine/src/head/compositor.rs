//! The compositor handles management of the swapchain and compositing together
//! the frames from <3d> and vello

use std::sync::Arc;

use wgpu::*;
use winit::{event_loop::EventLoopWindowTarget, window::WindowBuilder};

use super::Head;
use crate::{
    head::{canvas::Canvas, scene::Scene},
    prelude::*,
};

impl Head {
    /// Attempt to create a new window and rendering context
    pub async fn new(target: &EventLoopWindowTarget<()>) -> Result<Head> {
        let window = WindowBuilder::new().with_title("VG Game").build(target)?;
        let window = Arc::new(window);

        let size = window.inner_size();
        info!(size.width, size.height, "Created new window");

        let instance = Instance::default();
        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .ok_or(anyhow!("No suitable graphics adapter"))?;

        let info = adapter.get_info();
        info!(name = info.name, driver = info.driver, backend = ?info.backend, "Selected graphics adapter");

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("vg"),
                    required_features: Features::empty(),
                    required_limits: Limits::default(), // Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
                },
                None,
            )
            .await?;

        let adapter = Arc::new(adapter);
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let canvas = Canvas::new(Arc::clone(&device), Arc::clone(&queue))?;

        let scene = Scene::new(
            Arc::new(instance),
            Arc::clone(&adapter),
            Arc::clone(&device),
            Arc::clone(&queue),
        )?;

        let mut head = Head {
            adapter,
            device,
            queue,
            window,
            surface,
            canvas,
            scene,
        };

        // needs initial configuration
        head.configure();

        Ok(head)
    }

    /// Update the swapchain
    pub fn configure(&mut self) {
        let (width, height) = self.window.inner_size().into();

        let config = self
            .surface
            .get_default_config(&self.adapter, width, height)
            .expect("Adapter doesn't support surface");

        // Don't configure surface for 0x0 size, it's illegal
        if config.width * config.height != 0 {
            self.surface.configure(&self.device, &config);
        }

        self.canvas.configure(config.format);
        self.scene.configure(config.format);

        debug!(format = ?config.format, present = ?config.present_mode, "Configured surface");
    }

    /// Try to get the current swapchain texture, or reconfigure if error
    fn acquire_surface(&mut self) -> Check<SurfaceTexture> {
        // TODO: Surface may be suboptimal
        match self.surface.get_current_texture() {
            Ok(surface) => Check::Pass(surface),
            Err(err) => {
                error!("Failed to acquire surface texture: {err}");
                self.configure();
                Check::Fail
            }
        }
    }

    /// Perform all rendering
    pub fn render_composite(&mut self) -> Nil {
        let surface = self.acquire_surface()?;

        // First render 3D content, then overlay 2D content
        self.scene.render(&surface.texture);
        self.canvas.render(&surface);

        // Flip the surface to the screen. End of (render) frame
        surface.present();
        Nil
    }
}
