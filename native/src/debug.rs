use std::{
    alloc::{GlobalAlloc, System},
    sync::{
        atomic::{AtomicUsize, Ordering::Relaxed},
        Arc,
    },
    time::{Duration, Instant},
};

use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::window::Window;

use tracing::debug;

pub struct RepaintSignal(pub Arc<Window>);

impl epi::RepaintSignal for RepaintSignal {
    fn request_repaint(&self) {
        self.0.request_redraw();
    }
}

pub struct DebugUi {
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
    pub tick_time: Duration,
    pub force_smooth: bool,
    pub runtime_name: String,
    pub smoothed_frames: usize,
}

impl DebugUi {
    pub fn new(window: Arc<Window>, rt_name: impl ToString) -> DebugUi {
        let size = window.inner_size();
        let platform = Platform::new(PlatformDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor(),
            font_definitions: Default::default(),
            style: Default::default(),
        });

        let repaint_signal = Arc::new(RepaintSignal(window.clone()));

        DebugUi {
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
            tick_time: Duration::from_millis(1),
            force_smooth: false,
            runtime_name: rt_name.to_string(),
            smoothed_frames: 0,
        }
    }

    pub fn print(&mut self, msg: String) {
        if self.logger {
            self.log.push(msg);
        }
    }
}

impl epi::App for DebugUi {
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        puffin::profile_function!();

        egui::Window::new("VG").show(ctx, |ui| {
            let last_draw = self.last_draw.elapsed();
            ui.label(format!("RT: {}", self.runtime_name));
            ui.label(format!(
                "Framerate: {:.2}fps / {:.2?}",
                1.0 / last_draw.as_secs_f32(),
                last_draw,
            ));
            ui.label(format!(
                "Tickrate: {:.2}tps / {:.2?}",
                1.0 / self.tick_time.as_secs_f32(),
                self.tick_time,
            ));
            ui.label(format!("{} Frames / Tick", self.smoothed_frames));

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

            ui.checkbox(&mut self.force_smooth, "Force smooth framerate");

            ui.spacing();

            ui.checkbox(&mut self.logger, "Game log");
            if ui.checkbox(&mut self.profiler, "Engine profiler").changed() {
                debug!("Puffin scopes: {:?}", self.profiler);
            }
        });

        puffin::set_scopes_on(self.profiler);

        if self.profiler {
            self.profiler = puffin_egui::profiler_window(ctx);
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
