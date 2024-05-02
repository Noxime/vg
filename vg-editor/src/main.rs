use anyhow::{anyhow, Result};
use egui::{Context, ViewportId};
use egui_wgpu::{
    renderer::ScreenDescriptor,
    wgpu::{
        Color, Instance, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
        StoreOp,
    },
    Renderer,
};
use egui_winit::{
    winit::{
        dpi::LogicalSize,
        event::{Event as WinitEvent, WindowEvent},
        event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
        window::WindowBuilder,
    },
    State,
};

mod tracing;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    let tracing = tracing::init();

    let event_loop: EventLoop<()> = EventLoopBuilder::new().build()?;
    let editor_window = WindowBuilder::new()
        .with_title("VG Editor")
        .with_inner_size(LogicalSize::new(1600, 900))
        .build(&event_loop)?;

    let instance = Instance::new(Default::default());
    let surface = unsafe { instance.create_surface(&editor_window)? };

    let adapter = instance
        .request_adapter(&Default::default())
        .await
        .ok_or(anyhow!("No graphics adapter"))?;

    let (device, queue) = adapter.request_device(&Default::default(), None).await?;

    let size = editor_window.inner_size();
    let mut surface_config = surface
        .get_default_config(&adapter, size.width as u32, size.height as u32)
        .ok_or(anyhow!("GPU does not support window"))?;

    surface.configure(&device, &surface_config);

    // egui
    let mut renderer = Renderer::new(&device, surface_config.format, None, 1);
    let egui_ctx = Context::default();
    let mut egui_state = State::new(
        Default::default(),
        ViewportId::ROOT,
        &editor_window,
        None,
        None,
    );
    let mut ui = ui::EditorUi::new(tracing);

    event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);

        match &event {
            WinitEvent::WindowEvent { window_id, event } => {
                // This is an event for our editor window
                if *window_id == editor_window.id() {
                    match event {
                        WindowEvent::Resized(size) => {
                            if size.width != 0 && size.height != 0 {
                                surface_config.width = size.width as u32;
                                surface_config.height = size.height as u32;
                                surface.configure(&device, &surface_config);
                            }
                        }
                        WindowEvent::CloseRequested => target.exit(),
                        WindowEvent::RedrawRequested => {
                            // Repaint editor
                            let new_input = egui_state.take_egui_input(&editor_window);
                            let full_output = egui_ctx.run(new_input, |ctx| ui.update(ctx));
                            let clipped = egui_ctx
                                .tessellate(full_output.shapes, full_output.pixels_per_point);

                            egui_state.handle_platform_output(
                                &editor_window,
                                full_output.platform_output,
                            );

                            // Update resources
                            for id in full_output.textures_delta.free {
                                renderer.free_texture(&id);
                            }
                            for (id, delta) in full_output.textures_delta.set {
                                renderer.update_texture(&device, &queue, id, &delta);
                            }

                            let mut encoder = device.create_command_encoder(&Default::default());

                            let screen = ScreenDescriptor {
                                size_in_pixels: [surface_config.width, surface_config.height],
                                pixels_per_point: editor_window.scale_factor() as f32,
                            };

                            renderer.update_buffers(
                                &device,
                                &queue,
                                &mut encoder,
                                &clipped,
                                &screen,
                            );

                            // Begin rendering
                            let Ok(texture) = surface.get_current_texture() else {
                                return;
                            };
                            let view = texture.texture.create_view(&Default::default());

                            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                                label: Some("Editor egui"),
                                color_attachments: &[Some(RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: Operations {
                                        load: LoadOp::Clear(Color::BLUE),
                                        store: StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });

                            renderer.render(&mut pass, &clipped, &screen);
                            drop(pass);

                            queue.submit([encoder.finish()]);
                            texture.present();
                        }
                        _ => (),
                    }

                    let response = egui_state.on_window_event(&editor_window, &event);
                    if response.repaint {
                        editor_window.request_redraw();
                    }
                }
            }
            WinitEvent::AboutToWait => {
                editor_window.request_redraw();
            }
            _ => {}
        };

        // Pass events to all engines. They will filter their own events correctly
        ui.event(&event, target);
    })?;

    Ok(())
}
