use std::{
    alloc::{GlobalAlloc, System},
    sync::{
        atomic::{AtomicUsize, Ordering::Relaxed},
        Arc,
    },
    time::Instant,
};

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
    pub profiler: bool,
    pub platform: Platform,
    pub repaint_signal: Arc<RepaintSignal>,
    pub last_frame_time: Option<f32>,
    pub last_alloc: usize,
    pub last_dealloc: usize,
    pub log: Vec<String>,
    pub logger: bool,
    pub last_draw: Instant,
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
            profiler: false,
            logger: false,
            platform,
            repaint_signal,
            last_frame_time: None,
            last_alloc: 0,
            last_dealloc: 0,
            log: vec![],
            last_draw: Instant::now(),
        }
    }

    pub fn print(&mut self, msg: String) {
        if self.logger {
            self.log.push(msg);
        }
    }
}

impl epi::App for DebugData {
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        puffin::profile_function!();

        egui::Window::new("VG").show(ctx, |ui| {
            let last_draw = self.last_draw.elapsed();
            ui.label(format!(
                "Framerate: {:.2}fps / {:?}",
                1.0 / last_draw.as_secs_f32(),
                last_draw,
            ));

            ui.spacing();

            let (alloc, dealloc) = GLOBAL.load();
            ui.label(format!("Heap usage: {}", Memory(alloc - dealloc)));
            ui.label(format!(
                "Heap growth: {} / frame",
                Memory(alloc - self.last_alloc)
            ));
            ui.label(format!(
                "Heap shrink: {} / frame",
                Memory(dealloc - self.last_dealloc)
            ));

            self.last_alloc = alloc;
            self.last_dealloc = dealloc;

            ui.checkbox(&mut self.logger, "Game log");
            if ui.checkbox(&mut self.profiler, "Engine profiler").changed() {
                puffin::set_scopes_on(self.profiler);
            }
        });

        if self.profiler {
            puffin_egui::profiler_window(ctx);
        }

        if self.logger {
            egui::Window::new("Logger").show(ctx, |ui| {
                for line in self.log.drain(..) {
                    ui.monospace(line);
                }
            });
        }

        self.last_draw = Instant::now();
    }

    fn name(&self) -> &str {
        "VG-debug"
    }
}

struct Memory(usize);

impl std::fmt::Display for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const KB: usize = 1024;
        const MB: usize = 1024 * KB;
        const GB: usize = 1024 * MB;

        #[allow(overlapping_range_endpoints)]
        match self.0 {
            0..=KB => write!(f, "{} bytes", self.0),
            MB..=GB => write!(f, "{} MB", self.0 / MB),
            KB..=MB => write!(f, "{} KB", self.0 / KB),
            _ => write!(f, "{} GB", self.0 / GB),
        }
    }
}

struct TrackerAlloc {
    allocated: AtomicUsize,
    deallocated: AtomicUsize,
}

impl TrackerAlloc {
    fn load(&self) -> (usize, usize) {
        (self.allocated.load(Relaxed), self.deallocated.load(Relaxed))
    }
}

#[global_allocator]
static GLOBAL: TrackerAlloc = TrackerAlloc {
    allocated: AtomicUsize::new(0),
    deallocated: AtomicUsize::new(0),
};

unsafe impl GlobalAlloc for TrackerAlloc {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        self.allocated.fetch_add(layout.size(), Relaxed);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        self.deallocated.fetch_add(layout.size(), Relaxed);
        System.dealloc(ptr, layout)
    }
}
