use anyhow::{anyhow, Result};
use clap::Parser;
use egui::Context;
use egui_wgpu::{
    renderer::ScreenDescriptor,
    wgpu::{Color, Instance, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor},
    Renderer,
};
use egui_winit::{
    winit::{
        event::{Event as WinitEvent, WindowEvent},
        event_loop::{EventLoop, EventLoopBuilder},
        window::WindowBuilder,
    },
    State,
};
use pollster::block_on;
use std::path::PathBuf;

mod tracing;
mod ui;

#[derive(Parser)]
struct Args {
    /// Path to WebAssembly module
    path: PathBuf,
    /// URL of signaling server
    #[arg(long, default_value = "ws://vg.noxim.xyz:3536/")]
    url: String,
}

enum EditorEvent {}

fn main() -> Result<()> {
    let args = Args::parse();
    let tracing = tracing::init();

    let event_loop: EventLoop<EditorEvent> = EventLoopBuilder::with_user_event().build();
    let editor_window = WindowBuilder::new()
        .with_title("VG Editor")
        .build(&event_loop)?;

    let instance = Instance::new(Default::default());
    let surface = unsafe { instance.create_surface(&editor_window)? };

    let adapter = block_on(instance.request_adapter(&Default::default()))
        .ok_or(anyhow!("No graphics adapter"))?;

    let (device, queue) = block_on(adapter.request_device(&Default::default(), None))?;

    let size = editor_window.inner_size();
    let mut surface_config = surface
        .get_default_config(&adapter, size.width as u32, size.height as u32)
        .ok_or(anyhow!("GPU does not support window"))?;

    surface.configure(&device, &surface_config);

    // egui
    let mut renderer = Renderer::new(&device, surface_config.format, None, 1);
    let mut egui_state = State::new(&editor_window);
    let egui_ctx = Context::default();
    let mut ui = ui::EditorUi::new(tracing);

    event_loop.run(move |event, _target, control_flow| {
        control_flow.set_poll();

        match event {
            WinitEvent::WindowEvent { window_id, event } => {
                // This is an event for our editor window
                if window_id == editor_window.id() {
                    match event {
                        WindowEvent::Resized(size) => {
                            if size.width != 0 && size.height != 0 {
                                surface_config.width = size.width as u32;
                                surface_config.height = size.height as u32;
                                surface.configure(&device, &surface_config);
                            }
                        }
                        WindowEvent::CloseRequested => control_flow.set_exit(),
                        _ => (),
                    }

                    let response = egui_state.on_event(&egui_ctx, &event);
                    if response.repaint {
                        editor_window.request_redraw();
                    }
                }
            }
            WinitEvent::RedrawRequested(window_id) => {
                if window_id == editor_window.id() {
                    // Repaint editor
                    let new_input = egui_state.take_egui_input(&editor_window);
                    let full_output = egui_ctx.run(new_input, |ctx| ui.update(ctx));
                    let clipped = egui_ctx.tessellate(full_output.shapes);

                    egui_state.handle_platform_output(
                        &editor_window,
                        &egui_ctx,
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

                    renderer.update_buffers(&device, &queue, &mut encoder, &clipped, &screen);

                    // Begin rendering
                    let Ok(texture) = surface.get_current_texture() else { return };
                    let view = texture.texture.create_view(&Default::default());

                    let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: Some("Editor egui"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(Color::BLUE),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });

                    renderer.render(&mut pass, &clipped, &screen);
                    drop(pass);

                    queue.submit([encoder.finish()]);
                    texture.present();
                }
            }
            WinitEvent::RedrawEventsCleared => {
                editor_window.request_redraw();
            }
            _ => {}
        };
    })
}
