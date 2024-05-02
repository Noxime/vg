use std::{num::NonZeroUsize, sync::Arc};

use wgpu::*;

use crate::prelude::*;
use vello::{AaConfig, AaSupport, RenderParams, Renderer, RendererOptions};

pub struct Canvas {
    device: Arc<Device>,
    queue: Arc<Queue>,
    renderer: Renderer,
    format: TextureFormat,
}

impl Canvas {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Result<Canvas> {
        // Default
        let format = TextureFormat::Rgba8Unorm;

        let renderer = Renderer::new(
            &device,
            RendererOptions {
                surface_format: Some(format),
                use_cpu: false,
                antialiasing_support: AaSupport::area_only(),
                num_init_threads: NonZeroUsize::new(1), // TODO: No threading
            },
        )
        .map_err(|e| anyhow!("Vello initialization error: {e}"))?;

        Ok(Canvas {
            device,
            queue,
            renderer,
            format,
        })
    }

    pub fn configure(&mut self, format: TextureFormat) {
        // No need to recreate if just a resize reconfig
        if format == self.format {
            return;
        }
        self.format = format;

        self.renderer = Renderer::new(
            &self.device,
            RendererOptions {
                surface_format: Some(format),
                use_cpu: false,
                antialiasing_support: AaSupport::area_only(),
                num_init_threads: NonZeroUsize::new(1),
            },
        )
        .map_err(|e| anyhow!("Vello configuration error: {e}"))
        .unwrap();
    }

    pub fn render(&mut self, surface: &SurfaceTexture) {
        let mut scene = vello::Scene::new();

        scene.fill(
            vello::peniko::Fill::NonZero,
            vello::kurbo::Affine::IDENTITY,
            vello::peniko::Color::rgb8(242, 140, 168),
            None,
            &vello::kurbo::Circle::new((420.0, 200.0), 120.0),
        );

        self.renderer
            .render_to_surface(
                &self.device,
                &self.queue,
                &scene,
                surface,
                &RenderParams {
                    base_color: vello::peniko::Color::TRANSPARENT,
                    width: surface.texture.width(),
                    height: surface.texture.height(),
                    antialiasing_method: AaConfig::Area,
                },
            )
            .unwrap();
    }
}
