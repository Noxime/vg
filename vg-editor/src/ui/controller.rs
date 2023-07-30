use std::sync::Arc;

use egui::{TextEdit, Ui};
use egui_winit::winit::{event::Event, event_loop::EventLoopWindowTarget};
use vg_asset::FileSource;
use vg_engine::{Engine, EngineConfig};
pub struct Controller {
    engine: Lifecycle,
}

enum Lifecycle {
    Dead(EngineConfig),
    Live(Engine),
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            engine: Lifecycle::Dead(EngineConfig::new()),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        match &mut self.engine {
            Lifecycle::Dead(config) => {
                ui.checkbox(&mut config.headless, "Headless");
                ui.add(TextEdit::singleline(&mut config.path).code_editor());

                if ui.button("Start").clicked() {
                    let engine = Engine::with_config(config.clone());

                    let assets = Arc::clone(engine.assets());
                    tokio::spawn(FileSource::run(assets, "."));

                    self.engine = Lifecycle::Live(engine);
                }
            }
            Lifecycle::Live(engine) => {
                ui.label("Pending assets");
                for path in engine.assets().missing() {
                    ui.monospace(path.to_string_lossy());
                }

                ui.checkbox(&mut engine.config_mut().running, "Execute");

                if ui.button("Stop").clicked() || !engine.alive() {
                    self.engine = Lifecycle::Dead(engine.config_mut().clone());
                }
            }
        }
    }

    pub fn event(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {
        if let Lifecycle::Live(engine) = &mut self.engine {
            engine.event(event, target);
        }
    }
}
