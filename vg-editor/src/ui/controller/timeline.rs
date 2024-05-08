use std::ops::Range;

use egui::*;

pub struct Timeline {
    range: Range<f32>,
}

impl Timeline {
    pub fn new(range: Range<f32>) -> Self {
        Self { range }
    }
}

impl Widget for Timeline {
    fn ui(self, ui: &mut Ui) -> Response {
        Frame::canvas(ui.style()).show(ui, |ui| {
            let timeline_size = Vec2::new(ui.available_width(), 40.0);
            let (response, painter) = ui.allocate_painter(timeline_size, Sense::click_and_drag());

        }).response
    }
}
