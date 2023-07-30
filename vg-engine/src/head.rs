//! Rendering and presentation related functionality

use anyhow::Result;
use three_d::{
    camera2d, ClearState, Color, ColorMaterial, Gm, HasContext, Line, Object, RenderTarget,
    Viewport, WindowedContext,
};
use tracing::{debug, error, info};
use vg_interface::Draw;
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder, WindowId},
};

use crate::Engine;

pub struct Head {
    window: Window,
    context: WindowedContext,
}

impl Head {
    /// Attempt to create a new window and rendering context
    pub fn new(target: &EventLoopWindowTarget<()>) -> Result<Head> {
        let window = WindowBuilder::new().with_title("VG Game").build(target)?;
        let context = WindowedContext::from_winit_window(&window, Default::default())?;

        let size = window.inner_size();
        let version = context.version();
        info!(
            size.width,
            size.height, version.major, version.minor, version.vendor_info, "Created new window"
        );

        Ok(Head { window, context })
    }
}

impl Engine {
    /// Check if the ID matches our window. False if headless
    pub fn is_my_window(&self, id: &WindowId) -> bool {
        self.head.as_ref().map(|w| w.window.id()).as_ref() == Some(id)
    }

    /// Create a window if the current one is closed, unless in headless mode
    pub fn ensure_window(&mut self, target: &EventLoopWindowTarget<()>) {
        if !self.config.headless && self.head.is_none() && self.between_resumes {
            self.head = match Head::new(target) {
                Ok(w) => Some(w),
                Err(e) => {
                    error!("Failed to create window: {e}");
                    None
                }
            };
        }
    }

    fn with_head(&mut self, mut f: impl FnMut(&mut Head) -> Result<()>) {
        if let Some(head) = &mut self.head {
            if let Err(e) = f(head) {
                error!("Head error: {e}");
            }
        }
    }

    pub fn render(&mut self) {
        let scene = self.scene.clone();
        self.with_head(|head| {
            let (width, height) = head.window.inner_size().into();
            let camera = camera2d(Viewport {
                x: 0,
                y: 0,
                width,
                height,
            });

            let mut lines = vec![];
            for draw in &scene.draws {
                match draw {
                    Draw::Line {
                        color: (r, g, b, a),
                        points,
                    } => {
                        for ends in points.windows(2) {
                            let line = Gm::new(
                                Line::new(&head.context, ends[0].into(), ends[1].into(), 5.0),
                                ColorMaterial {
                                    color: Color::from_rgba_slice(&[*r, *g, *b, *a]),
                                    ..Default::default()
                                },
                            );

                            lines.push(line);
                        }
                    }
                }
            }


            RenderTarget::screen(&head.context, width, height)
                .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0))
                .render(&camera, lines, &[]);

            head.context.swap_buffers()?;
            Ok(())
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.with_head(|head| {
            head.context.resize(size);
            debug!(width = size.width, height = size.height, "Resized window");
            Ok(())
        })
    }

    pub fn redraw(&mut self) {
        self.with_head(|head| Ok(head.window.request_redraw()))
    }
}
