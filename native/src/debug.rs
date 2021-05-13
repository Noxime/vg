use std::sync::Arc;

use egui::mutex::Mutex;
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::window::Window;

pub struct RepaintSignal(pub Arc<Window>);

impl epi::RepaintSignal for RepaintSignal {
    fn request_repaint(&self) {
        self.0.request_redraw();
    }
}

pub struct DebugData {
    pub visible: bool,
    pub platform: Platform,
    pub repaint_signal: Arc<RepaintSignal>,
    pub last_frame_time: Option<f32>,
}

impl DebugData {
    pub fn new(window: Arc<Window>) -> DebugData {
        let size = window.inner_size();
        let platform = Platform::new(PlatformDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor(),
            font_definitions: Default::default(),
            style: Default::default(),
        });

        let repaint_signal = Arc::new(RepaintSignal(window.clone()));

        DebugData {
            visible: false,
            platform,
            repaint_signal,
            last_frame_time: None,
        }
    }
}

impl epi::App for DebugData {
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        egui::Window::new("Log").show(ctx, |ui| {
            ui.label("Hi");
        });
    }

    fn name(&self) -> &str {
        "VG-debug"
    }
}
