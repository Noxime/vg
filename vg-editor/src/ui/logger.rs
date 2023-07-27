use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use chrono::{DateTime, Local};
use egui::{epaint::text::TextWrapping, text::LayoutJob, *};
use egui_extras::{Column, TableBuilder};
use tracing::Level;

use crate::tracing::{LevelMask, Tracing, TracingEvent};

pub struct Logger {
    id: usize,
    tracing: Arc<Tracing>,
    selected: Option<usize>,
    auto_scroll: bool,
    hide: bool,
    mask: LevelMask,
}

impl Logger {
    pub fn new(tracing: Arc<Tracing>) -> Logger {
        static UNIQUE: AtomicUsize = AtomicUsize::new(0);

        Logger {
            id: UNIQUE.fetch_add(1, Ordering::Relaxed),
            tracing,
            selected: None,
            auto_scroll: true,
            hide: true,
            mask: LevelMask::all(),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        // Logger
        ui.vertical(|ui| {
            // Event inspector
            TopBottomPanel::bottom(format!("Inspector {}", self.id))
                .resizable(true)
                .show_animated_inside(ui, self.selected.is_some(), |ui| {
                    if let Some(i) = self.selected {
                        self.tracing.with(i, |event| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    if ui.button("Close").clicked() {
                                        self.selected = None;
                                    }
                                    ui.checkbox(&mut self.hide, "Hide reduntant");

                                    ui.label(format!("Time: {}", format_time(event.time)));
                                    ui.label(format!("Level: {}", event.metadata.level().as_str()));
                                    ui.label(format!("Name: {}", event.metadata.name().as_str()));
                                    ui.label(format!("Target: {}", event.metadata.target()));
                                });

                                ui.vertical(|ui| {
                                    let hidden = ["log.file", "log.line", "log.target"];

                                    for (name, value) in event.fields() {
                                        if self.hide && hidden.contains(&name) {
                                            continue;
                                        }
                                        ui.label(format!("{name}={value}"));
                                    }
                                });
                            });
                        });
                    }
                });

            // Event list
            CentralPanel::default().show_inside(ui, |ui| {
                let height = ui.available_height();
                let mut builder = TableBuilder::new(ui)
                .auto_shrink([false, false])
                .max_scroll_height(height)
                .striped(true)
                .column(Column::exact(80.0)) //  Time
                .column(Column::exact(50.0)) // Level
                .column(
                    // Target
                    Column::initial(120.0)
                        .at_least(40.0)
                        .resizable(true)
                        .clip(true),
                )
                .column(Column::remainder().clip(true)) // Message
                ;

                if self.auto_scroll && self.tracing.len() != 0 {
                    builder = builder.scroll_to_row(self.tracing.len(), None);
                }

                builder
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.checkbox(&mut self.auto_scroll, "Time")
                                .on_hover_text("Auto-scroll to latest");
                        });
                        header.col(|ui| {
                            ui.menu_button("Level", |ui| {
                                ui.checkbox(&mut self.mask.trace, format_level(Level::TRACE));
                                ui.checkbox(&mut self.mask.debug, format_level(Level::DEBUG));
                                ui.checkbox(&mut self.mask.info, format_level(Level::INFO));
                                ui.checkbox(&mut self.mask.warn, format_level(Level::WARN));
                                ui.checkbox(&mut self.mask.error, format_level(Level::ERROR));
                            });
                        });
                        header.col(|ui| {
                            ui.label("Target");
                        });
                        header.col(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("Events: {}", self.tracing.len()));
                                if ui.button("Clear").clicked() {
                                    self.tracing.clear();
                                }
                            });
                        });
                    })
                    .body(|body| {
                        body.rows(20.0, self.tracing.len(), |i, mut row| {
                            self.tracing.with(i, |event| {
                                row.col(|ui| {
                                    ui.label(format_time(event.time));
                                });
                                row.col(|ui| {
                                    ui.label(format_level(*event.metadata.level()));
                                });
                                row.col(|ui| {
                                    let target = event.metadata.target().to_string();
                                    ui.label(wrap(target.clone())).on_hover_text(&target);
                                });
                                row.col(|ui| {
                                    if ui
                                        .selectable_label(
                                            self.selected == Some(i),
                                            format_fields(event),
                                        )
                                        .clicked()
                                    {
                                        self.selected = Some(i);
                                    }
                                });
                            });
                        })
                    });
            });
        });
    }
}

fn wrap(s: String) -> LayoutJob {
    let mut job = LayoutJob::single_section(s, Default::default());
    job.wrap = TextWrapping {
        max_rows: 1,
        break_anywhere: true,
        ..Default::default()
    };
    job.break_on_newline = false;

    job
}

fn format_time(time: DateTime<Local>) -> String {
    time.format("%T%.3f").to_string()
}

fn format_level(level: Level) -> RichText {
    let text = level.as_str();
    let color = match level {
        Level::TRACE => Color32::from_rgb(127, 127, 255),
        Level::DEBUG => Color32::from_rgb(255, 127, 255),
        Level::INFO => Color32::from_rgb(127, 255, 127),
        Level::WARN => Color32::from_rgb(255, 255, 127),
        Level::ERROR => Color32::from_rgb(255, 127, 127),
    };

    RichText::new(text).color(color)
}

fn format_fields<'a>(event: &TracingEvent) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap = TextWrapping {
        max_rows: 1,
        break_anywhere: true,
        ..Default::default()
    };
    job.break_on_newline = false;

    // Print message
    if let Some((_, value)) = event.fields().find(|m| m.0 == "message") {
        job.append(value, 0.0, TextFormat::default());
    }

    for (name, value) in event.fields() {
        if name != "message" {
            let mut format = TextFormat::default();
            format.italics = true;
            format.color = format.color.gamma_multiply(0.75);
            job.append(&format!("{name}={value}"), 2.0, format);
        }
    }

    job
}
