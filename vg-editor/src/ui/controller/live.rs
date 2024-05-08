use std::{
    ops::{Range, RangeInclusive},
    sync::Arc,
};

use anyhow::anyhow;
use egui::{Align2, Button, Color32, DragValue, Rect, ScrollArea, Sense, Ui, Vec2};
use egui_winit::winit::{event::Event, event_loop::EventLoopWindowTarget};
use vg_asset::FileSource;
use vg_engine::*;

/// Represents an instantiated... instance.. of a vg engine
pub struct Live {
    engine: Engine,
    goto: Option<RuntimeInstant>,
    history: Vec<SaveState>,
    max_reached: RuntimeInstant,
    scale: f32,
}

impl Live {
    pub fn from_config(config: EngineConfig) -> Live {
        let engine = Engine::with_config(config.clone());

        let assets = Arc::clone(engine.assets());
        tokio::spawn(FileSource::run(assets, "."));

        Live {
            engine,
            goto: None,
            history: vec![],
            max_reached: RuntimeInstant::EPOCH,
            scale: 8.0,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Option<EngineConfig> {
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

        self.history_ui(ui);

        if ui.button("End").clicked() || !self.engine.alive() {
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
        // Try to make sure there is always one save in the history
        if self.history.is_empty() {
            self.push_save();
        }

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
                ) >= 1000
                {
                    self.push_save();
                }

                // Roll back to previous state
                if let Some(goto) = self.goto {
                    self.goto_save(goto).unwrap();
                }
            }
        }

        self.max_reached = self.max_reached.max(self.engine.runtime_instant());
    }

    /// Inserts the current save state into it's chronological position
    #[profiling::function]
    fn push_save(&mut self) {
        let Some(save) = self.engine.save_state() else {
            return;
        };

        // Remove all saves that would be after this
        // self.history.retain(|old| old.instant() < save.instant());

        // Guaranteed to be in chronological order
        self.history.push(save);
    }

    /// Goes to the history and loads the right savepoint and then steps forward
    /// until requested instant is matched
    #[profiling::function]
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

        tracing::trace!(
            "Went to state at {instant}, from {}",
            self.history[parent].instant()
        );

        Ok(())
    }

    /// Draw the timeline of frames, highlighting saved ones
    #[profiling::function]
    fn history_ui(&mut self, ui: &mut Ui) {
        let latest: RuntimeInstant = self.engine.runtime_instant();
        let oldest = self
            .history
            .first()
            .map(SaveState::instant)
            .unwrap_or(RuntimeInstant::EPOCH);

        let accessible_range = oldest..=self.max_reached;
        let accessible_frames = RangeInclusive::new(
            accessible_range.start().frames_since(RuntimeInstant::EPOCH),
            accessible_range.end().frames_since(RuntimeInstant::EPOCH),
        );

        ui.horizontal(|ui| {
            let now = self.engine.runtime_instant();
            let goto = &mut self.goto;

            if ui
                .button(match goto {
                    None => "⏸",
                    Some(_) => "⏯",
                })
                .clicked()
            {
                match goto {
                    None => *goto = Some(now),
                    Some(_) => *goto = None,
                }
            }

            if ui.add_enabled(goto.is_some(), Button::new("⏴")).clicked() {
                *goto = Some(goto.unwrap().prev_frame());
            }

            let mut goto_frame = goto.unwrap_or(now).frames_since(RuntimeInstant::EPOCH);
            ui.add_enabled(
                goto.is_some(),
                DragValue::new(&mut goto_frame).clamp_range(accessible_frames.clone()),
            );

            if goto.is_some() {
                *goto = Some(RuntimeInstant::EPOCH.relative_frame(goto_frame))
            }

            if ui.add_enabled(goto.is_some(), Button::new("⏵")).clicked() {
                *goto = Some(goto.unwrap().next_frame());
            }
        });

        // Apply zooming
        ui.input(|i| self.scale *= i.zoom_delta());
        self.scale = self.scale.clamp(1.0, 64.0);

        ScrollArea::horizontal()
            .stick_to_right(true)
            .show_viewport(ui, |ui, viewport| {
                let width = self.scale;
                let padded_width = width + 1.0;

                let (response, painter) = ui.allocate_painter(
                    Vec2 {
                        x: accessible_frames.count() as f32 * padded_width,
                        y: 40.0,
                    },
                    Sense::click(),
                );

                let range = {
                    let min = (viewport.min.x / padded_width).floor() as usize;
                    let max = (viewport.max.x / padded_width).ceil() as usize;
                    min..=max
                };

                for i in range.clone() {
                    let pos = response.rect.min + Vec2::new(i as f32 * padded_width, 0.0);
                    let rect = Rect::from_min_size(pos, Vec2::new(width, 16.0));

                    let instant = oldest.relative_frame(i as isize);

                    let save = self
                        .history
                        .binary_search_by_key(&instant, SaveState::instant)
                        .ok();

                    let is_hovered = if let Some(hover) = response.hover_pos() {
                        rect.contains(hover)
                    } else {
                        false
                    };

                    if is_hovered {
                        response.clone().on_hover_ui_at_pointer(|ui| {
                            ui.label(format!("{instant}"));
                        });
                    }

                    if is_hovered && response.clicked() {
                        self.goto = Some(instant);
                    }

                    let color = if is_hovered {
                        Color32::LIGHT_GRAY
                    } else if instant == latest {
                        Color32::LIGHT_BLUE
                    } else if save.is_some() {
                        Color32::LIGHT_GREEN
                    } else if self.saved_range().contains(&instant) {
                        Color32::GRAY
                    } else {
                        Color32::DARK_GRAY
                    };

                    painter.rect_filled(rect, 2.0, color);
                }

                let text_stride = (64.0 / width).ceil() as usize;
                let text_range = RangeInclusive::new(range.start() - range.start() % text_stride, *range.end());
                for i in text_range.step_by(text_stride) {
                    let pos = response.rect.min + Vec2::new(i as f32 * padded_width, 16.0);

                    let instant = oldest.relative_frame(i as isize);

                    painter.line_segment([pos, pos + Vec2::new(0.0, 16.0)], (1.0, Color32::GRAY));

                    painter.text(
                        pos + Vec2::new(4.0, 0.0),
                        Align2::LEFT_TOP,
                        format!("{instant}"),
                        Default::default(),
                        ui.visuals().text_color(),
                    );
                }
            });
    }

    /// Time range we have of rollback saves
    fn saved_range(&self) -> Range<RuntimeInstant> {
        let first = self.history.first().map(SaveState::instant);
        let last = self.history.last().map(SaveState::instant);
        match (first, last) {
            (None, None) => RuntimeInstant::EPOCH..RuntimeInstant::EPOCH,
            (Some(one), None) | (None, Some(one)) => one..one,
            (Some(first), Some(last)) => first..last,
        }
    }
}
