use std::sync::Arc;

use anyhow::anyhow;
use egui::{Color32, Frame, ScrollArea};
use egui_extras::{Column, TableBuilder};
use egui_winit::winit::{event::Event, event_loop::EventLoopWindowTarget};
use vg_asset::FileSource;
use vg_engine::*;

/// Represents an instantiated... instance.. of a vg engine
pub struct Live {
    engine: Engine,
    pause: bool,
    history: Vec<SaveState>,
}

impl Live {
    pub fn from_config(config: EngineConfig) -> Live {
        let engine = Engine::with_config(config.clone());

        let assets = Arc::clone(engine.assets());
        tokio::spawn(FileSource::run(assets, "."));

        Live {
            engine,
            pause: false,
            history: vec![],
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<EngineConfig> {
        ui.collapsing("Assets", |ui| {
            ui.label("Pending:");
            for path in self.engine.assets().missing() {
                ui.monospace(path.to_string_lossy());
            }
            ui.label("Available:");
            for path in self.engine.assets().available() {
                ui.monospace(path.to_string_lossy());
            }
        });

        ui.checkbox(&mut self.pause, "Pause");

        let now = self.engine.runtime_instant();
        ui.label(format!("At {now}"));
        if self.pause {
            if ui.button("Previous").clicked() {
                self.goto_save(now.prev_frame()).unwrap();
            }

            if ui.button("Next").clicked() {
                self.goto_save(now.next_frame()).unwrap();
            }
        }

        TableBuilder::new(ui)
            .column(Column::auto().at_least(100.0))
            .column(Column::remainder())
            .resizable(true)
            .stick_to_bottom(true)
            .body(|body| {
                body.rows(14.0, self.history.len(), |mut row| {
                    let save = self.history[row.index()].instant();

                    row.col(|ui| {
                        ui.label(format!("Save: {save}"));
                    });
                    row.col(|ui| {
                        if ui.button("Restore").clicked() {
                            self.goto_save(save).unwrap();
                        }
                    });
                })
            });

        if ui.button("Stop").clicked() || !self.engine.alive() {
            return Some(self.engine.config_mut().clone());
        }

        None
    }

    #[profiling::function]
    pub fn event(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {
        self.engine.event(event, target);
    }

    #[profiling::function]
    pub fn poll(&mut self) {
        let before_instant = self.engine.runtime_instant();

        // Potentially run a single step forward
        match self.engine.poll() {
            PollResult::None => (),
            // State has advanced
            PollResult::Tick => {
                if self.engine.runtime_instant().frames_since(
                    self.history
                        .last()
                        .map(SaveState::instant)
                        .unwrap_or(RuntimeInstant::EPOCH),
                ) > 100
                {
                    self.push_save();
                }

                // Roll back to previous state
                if self.pause {
                    self.goto_save(before_instant).unwrap();
                }
            }
        }
    }

    /// Inserts the current save state into it's chronological position
    fn push_save(&mut self) {
        let Some(save) = self.engine.save_state() else {
            return;
        };

        // Remove all saves that would be after this
        self.history.retain(|old| old.instant() < save.instant());

        // Guaranteed to be in chronological order
        self.history.push(save);
    }

    /// Goes to the history and loads the right savepoint and then steps forward
    /// until requested instant is matched
    fn goto_save(&mut self, instant: RuntimeInstant) -> anyhow::Result<()> {
        if self.history.is_empty() {
            return Err(anyhow!("No history"));
        };

        // Find a potential candidate
        let position = self
            .history
            .binary_search_by_key(&instant, SaveState::instant);

        // Find either perfect match, or the save just before
        let parent = match position {
            Ok(found) => found,
            Err(would_be) => would_be
                .checked_sub(1)
                .ok_or(anyhow!("History does not go that far"))?,
        };

        // Go to parent state
        self.engine.restore_state(&self.history[parent])?;

        assert!(self.engine.runtime_instant() <= instant);

        // Run until instant is found
        while self.engine.runtime_instant() < instant {
            self.engine.poll();
        }

        tracing::info!(
            "Went to state at {instant}, from {}",
            self.history[parent].instant()
        );

        Ok(())
    }
}
