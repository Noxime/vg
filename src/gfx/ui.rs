use crate::{Game, Network, Vg};
use egui::{app::*, *};

#[derive(Default)]
pub struct Ui {
    pub visible: bool,
    pub state: Option<String>,
}

impl<G: Game> App for Vg<G> {
    fn ui(
        &mut self,
        ctx: &std::sync::Arc<Context>,
        integration_context: &mut IntegrationContext<'_>,
    ) {
        if !self.debug_menu.visible {
            return;
        };

        SidePanel::left(Id::new(0), 250.0).show(ctx, |ui| {
            ui.heading("Debug");

            ui.monospace(format!("Tick rate: {} tps", self.options.tick_rate));
            ui.monospace(format!(
                "Prediction rate: {} tps",
                self.options.predict_rate
            ));

            ui.monospace(format!("Player ID: {}", self.options.tick_rate));

            match &self.network {
                Network::Host { socket, .. } => {
                    ui.monospace(format!("Host: {:?}", socket.local_addr().unwrap()));
                }
                Network::Client {
                    remote, round_trip, ..
                } => {
                    ui.monospace(format!(
                        "Client: {:?}",
                        remote.get_ref().peer_addr().unwrap(),
                    ));
                    ui.monospace(format!("RTT: {:?}", round_trip));
                }
            }

            let mut b = self.debug_menu.state.is_some();
            let c = b;
            ui.checkbox(&mut b, "State debugger");

            if c != b {
                if b {
                    self.debug_menu.state = serde_json::to_string_pretty(&self.state).ok();
                } else if let Some(s) = self
                    .debug_menu
                    .state
                    .take()
                    .and_then(|s| serde_json::from_str(&s).ok())
                {
                    self.state = s;
                }
            }

            if let Some(ref mut x) = self.debug_menu.state {
                ui.text_edit(x);
            }
        });

        integration_context.output.window_size = Some(ctx.used_size());
    }
}
