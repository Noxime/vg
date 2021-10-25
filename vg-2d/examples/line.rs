use std::{sync::Arc, time::Instant};

use anyhow::{anyhow, Result};
use glam::{UVec2, Vec2, Vec4};
use pollster::block_on;
use vg_2d::{calculate_bounds, RenderOutput, Renderer, Shape};
use wgpu::{
    Adapter, Backends, Device, Instance, Maintain, PresentMode, RequestAdapterOptions, Surface,
    SurfaceConfiguration, TextureFormat, TextureUsages,
};
use winit::{dpi::PhysicalSize, event::{Event, VirtualKeyCode, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::Window};
use winit_input_helper::WinitInputHelper;

fn main() -> Result<()> {
    emoji_logger::init();

    let events = EventLoop::new();
    let window = Window::new(&events)?;
    let mut helper = WinitInputHelper::new();

    let instance = Instance::new(Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .ok_or(anyhow!("No compatible graphics adapter"))?;
    let (device, queue) = block_on(adapter.request_device(&Default::default(), None))?;
    let device = Arc::new(device);

    println!("Rendering with: {}", adapter.get_info().name);

    let size = window.inner_size();
    let mut format = reconfigure(&surface, &device, &adapter, size);
    let mut bounds = calculate_bounds(UVec2::new(size.width, size.height));

    let mut renderer = Renderer::new(device.clone(), UVec2::new(size.width, size.height));

    let time = Instant::now();
    let mut focused = true;

    events.run(move |ev, _, flow| {
        *flow = ControlFlow::Poll;
        device.poll(Maintain::Poll);

        if let Event::WindowEvent { event: WindowEvent::Focused(x), .. } = ev {
            focused = x;
        }

        if helper.update(&ev) {
            if helper.quit() || helper.key_pressed(VirtualKeyCode::Escape) {
                *flow = ControlFlow::Exit;
            }

            if let Some(size) = helper.window_resized() {
                format = reconfigure(&surface, &device, &adapter, size);
                renderer.resize(UVec2::new(size.width, size.height));
                bounds = calculate_bounds(UVec2::new(size.width, size.height));
            }

            // MacOS Fix: WGPU leaks resources when rendering while a window is covered
            if !focused { return }

            let frame = surface.get_current_texture().unwrap();

            let t = time.elapsed().as_secs_f32();
            let shapes = vec![
                Shape::line(Vec2::new(-0.9, 0.4), Vec2::new(-0.5, 0.8))
                    .with_width(0.04 + t.sin() * 0.02)
                    .with_color(Vec4::new(1.0, 0.5, 0.5, 1.0)),
                Shape::line(Vec2::new(-0.2, 0.45), Vec2::new(0.2, 0.8))
                    .with_width(0.02)
                    .with_outline(Some(0.01 + t.sin() * 0.005))
                    .with_color(Vec4::new(0.5, 1.0, 0.5, 1.0)),
                Shape::circle(Vec2::new(-0.7, 0.0))
                    .with_width(0.16 + t.sin() * 0.04)
                    .with_color(Vec4::new(0.4, 0.8, 0.8, 1.0)),
                Shape::circle(Vec2::new(0.0, 0.0))
                    .with_width(0.12)
                    .with_outline(Some(0.04 + t.sin() * 0.02))
                    .with_color(Vec4::new(0.8, 0.4, 0.8, 1.0)),
                Shape::rect(Vec2::new(0.5, 0.6), Vec2::new(0.9, 0.6), 0.4)
                    .with_width(0.02 + t.sin() * 0.02)
                    .with_color(Vec4::new(0.5, 0.5, 1.0, 1.0)),
                Shape::rect(Vec2::new(0.5, 0.0), Vec2::new(0.9, 0.0), 0.4)
                    .with_width(0.02 + t.sin() * 0.02)
                    .with_outline(Some(0.01 + t.sin() * 0.005))
                    .with_color(Vec4::new(0.8, 0.8, 0.4, 1.0)),
                Shape::triangle(
                    Vec2::new(-0.9, -0.8),
                    Vec2::new(-0.7, -0.4),
                    Vec2::new(-0.5, -0.8),
                )
                .with_outline(Some(0.01 + t.sin() * 0.005)),
                Shape::bezier(
                    Vec2::new(-0.2, -0.8),
                    Vec2::new(t.sin() * 0.2, -0.4),
                    Vec2::new(t.cos() * 0.2, -0.4),
                    Vec2::new(0.2, -0.8),
                )
                .with_width(0.1)
            ];

            let output = RenderOutput {
                view: frame.texture.create_view(&Default::default()),
                format,
            };

            renderer.render(
                &queue,
                &shapes,
                Some(Vec4::new(0.0, 0.0, 0.0, 1.0)),
                output,
                bounds,
            );

            frame.present();
        }
    })
}

fn reconfigure(
    surface: &Surface,
    device: &Device,
    adapter: &Adapter,
    size: PhysicalSize<u32>,
) -> TextureFormat {
    let format = surface
        .get_preferred_format(&adapter)
        .unwrap_or(TextureFormat::Rgba8UnormSrgb);

    surface.configure(
        &device,
        &SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Mailbox,
        },
    );

    format
}
