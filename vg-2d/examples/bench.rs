use std::{sync::Arc, time::Instant};

use anyhow::{anyhow, Result};
use glam::{UVec2, Vec2, Vec4};
use pollster::block_on;
use rand::{thread_rng, Rng};
use vg_2d::{calculate_bounds, RenderOutput, Renderer, Shape};
use wgpu::{
    Adapter, Backends, Device, Instance, Maintain, PresentMode, RequestAdapterOptions, Surface,
    SurfaceConfiguration, TextureFormat, TextureUsages,
};
use winit::{
    dpi::PhysicalSize,
    event::VirtualKeyCode,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
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
    .ok_or_else(|| anyhow!("No compatible graphics adapter"))?;
    let (device, queue) = block_on(adapter.request_device(&Default::default(), None))?;
    let device = Arc::new(device);

    println!("Rendering with: {}", adapter.get_info().name);

    let size = window.inner_size();
    let mut format = reconfigure(&surface, &device, &adapter, size);
    let mut bounds = calculate_bounds(UVec2::new(size.width, size.height));

    let mut renderer = Renderer::new(device.clone(), UVec2::new(size.width, size.height));

    let mut shapes = vec![];
    let mut rng = thread_rng();

    let mut time = Instant::now();

    events.run(move |ev, _, flow| {
        *flow = ControlFlow::Poll;
        device.poll(Maintain::Poll);

        if helper.update(&ev) {
            if helper.quit() || helper.key_pressed(VirtualKeyCode::Escape) {
                *flow = ControlFlow::Exit;
            }

            if let Some(size) = helper.window_resized() {
                format = reconfigure(&surface, &device, &adapter, size);
                renderer.resize(UVec2::new(size.width, size.height));
                bounds = calculate_bounds(UVec2::new(size.width, size.height));
            }

            let frame = surface.get_current_texture().unwrap();

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

            let elapsed = time.elapsed();
            println!(
                "Took: {:?}, {:.2}/s",
                elapsed,
                shapes.len() as f32 / elapsed.as_secs_f32()
            );
            time += elapsed;

            shapes.clear();
            for _ in 0..(64 * 1024) {
                shapes.push(
                    Shape::circle(rng.gen::<Vec2>() * 2.0 - 1.0)
                        .with_radius(rng.gen_range(0.0..0.01))
                        .with_color(rng.gen()),
                );
            }
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
        .get_preferred_format(adapter)
        .unwrap_or(TextureFormat::Rgba8UnormSrgb);

    surface.configure(
        device,
        &SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Immediate,
        },
    );

    format
}
