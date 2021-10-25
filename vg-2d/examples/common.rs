use std::sync::Arc;

use glam::UVec2;
use pollster::block_on;
use wgpu::{
    Backends, Device, Instance, Maintain, PresentMode, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, TextureFormat, TextureUsages, TextureView,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use winit_input_helper::WinitInputHelper;

pub fn run<T: 'static>(
    init: impl Fn(Arc<Device>, UVec2, TextureFormat) -> T,
    draw: impl Fn(&mut T, Option<UVec2>, TextureView, &Queue) + 'static,
) -> ! {
    // Init winit
    let events = EventLoop::new();
    let window = Window::new(&events).expect("Failed to create window");
    let mut helper = WinitInputHelper::new();

    // Init WGPU
    let instance = Instance::new(Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .expect("No adapter available for surface");
    let (device, queue) = block_on(adapter.request_device(&Default::default(), None))
        .expect("Failed to request device");
    let device = Arc::new(device);

    let format = surface
        .get_preferred_format(&adapter)
        .unwrap_or(TextureFormat::Bgra8UnormSrgb);
    let size = window.inner_size();

    reconfigure(&surface, &device, size, format);

    println!("Rendering with: {}", adapter.get_info().name);

    // MacOS Fix: M1 Metal will not free any metal resources when window is obscured, thus running out of memory
    let mut focused = true;

    let mut data = init(device.clone(), UVec2::new(size.width, size.height), format);

    events.run(move |ev, _, flow| {
        *flow = ControlFlow::Poll;
        device.poll(Maintain::Poll);

        if let Event::WindowEvent {
            event: WindowEvent::Focused(x),
            ..
        } = ev
        {
            focused = x;
        }

        if !focused {
            return;
        }

        if helper.update(&ev) {
            if helper.quit() || helper.key_pressed(VirtualKeyCode::Escape) {
                *flow = ControlFlow::Exit;
            }


            let size = if let Some(size) = helper.window_resized() {
                reconfigure(&surface, &device, size, format);
                Some(UVec2::new(size.width, size.height))
            } else {
                None
            };

            let frame = surface.get_current_texture().unwrap();

            draw(
                &mut data,
                size,
                frame.texture.create_view(&Default::default()),
                &queue,
            );

            frame.present();
        }
    })
}

#[allow(dead_code)]
fn main() {
    eprintln!("This is not really an example, see shapes.rs or bench.rs");
}

fn reconfigure(surface: &Surface, device: &Device, size: PhysicalSize<u32>, format: TextureFormat) {
    surface.configure(
        device,
        &SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Mailbox,
        },
    )
}
