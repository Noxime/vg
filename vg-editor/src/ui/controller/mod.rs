use egui::{TextEdit, Ui};
use egui_winit::winit::{event::Event, event_loop::EventLoopWindowTarget};
use vg_engine::EngineConfig;

use self::live::Live;
mod live;
mod timeline;

pub struct Controller {
    lifecycle: Lifecycle,
}

enum Lifecycle {
    Dead(EngineConfig),
    Live(Live),
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            lifecycle: Lifecycle::Dead(EngineConfig::new()),
        }
    }

    #[profiling::function]
    pub fn ui(&mut self, ui: &mut Ui) {
        match &mut self.lifecycle {
            Lifecycle::Dead(config) => {
                // Networking config
                let mut networking = config.room.is_some();
                ui.checkbox(&mut networking, "Networking");
                if networking {
                    ui.label("Signaling server");
                    ui.add(TextEdit::singleline(&mut config.signaling).code_editor());
                    ui.label("Matchmaking room");
                    let mut room = config.room.clone().unwrap_or_default();
                    ui.add(TextEdit::singleline(&mut room).code_editor());
                    config.room = Some(room);
                } else {
                    config.room = None;
                }

                // Runtime execution
                ui.label("Game entrypoint");
                ui.add(TextEdit::singleline(&mut config.path).code_editor());

                // Presentation
                ui.checkbox(&mut config.headless, "Run in headless mode");

                if ui.button("Start").clicked() {
                    let live = Live::from_config(config.clone());
                    self.lifecycle = Lifecycle::Live(live);
                }
            }
            Lifecycle::Live(live) => {
                if let Some(config) = live.ui(ui) {
                    self.lifecycle = Lifecycle::Dead(config)
                }
            }
        }
    }

    #[profiling::function]
    pub fn event(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {
        if let Lifecycle::Live(live) = &mut self.lifecycle {
            live.event(event, target);
        };
    }

    #[profiling::function]
    pub fn poll(&mut self) {
        if let Lifecycle::Live(live) = &mut self.lifecycle {
            live.poll()
        }
    }
}
