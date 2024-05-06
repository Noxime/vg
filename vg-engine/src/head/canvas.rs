use std::num::NonZeroUsize;

use vg_interface::Draw;
use wgpu::*;

use crate::{prelude::*, runtime::WorldState};
use vello::{kurbo::Stroke, AaConfig, AaSupport, RenderParams, Renderer, RendererOptions, Scene};

pub struct Canvas {
    device: Arc<Device>,
    queue: Arc<Queue>,
    renderer: Renderer,
    format: TextureFormat,
}

#[profile_all]
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

    pub fn render(&mut self, surface: &SurfaceTexture, world: &WorldState) {
        let mut scene = Scene::new();

        for draw in &world.draws {
            match draw {
                Draw::Line {
                    color: (r, g, b, a),
                    points,
                } => {
                    for [x, y] in points.array_windows() {
                        scene.stroke(
                            &Stroke::new(1.0),
                            Default::default(),
                            vello::peniko::Color::rgba(*r as _, *g as _, *b as _, *a as _),
                            None,
                            &vello::kurbo::Line::new(
                                (x.0 as f64, x.1 as f64),
                                (y.0 as f64, y.1 as f64),
                            ),
                        );
                    }
                }
            }
        }

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
