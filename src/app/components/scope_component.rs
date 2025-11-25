use super::AppComponent;
use crate::app::App;
use crate::egui::epaint::*;
use crate::egui::{pos2, vec2, Frame, Pos2, Rect};

pub struct ScopeComponent;

impl AppComponent for ScopeComponent {
    type Context = App;
    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui) {
        Frame::canvas(ui.style()).show(ui, |ui| {
            ui.ctx().request_repaint();
            let _time = ui.input(|i| i.time);
            let color = Color32::from_additive_luminance(196);

            let desired_size = ui.available_width() * vec2(1.0, 0.25);
            let (_id, rect) = ui.allocate_space(desired_size);

            let to_screen =
                emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0), rect);
            let mut shapes = vec![];

            if let Some(ref mut scope) = &mut ctx.scope {
                if ctx.gui_num_bytes_read > 0 {
                    for sample in (ctx.ui_audio_buffer[0..ctx.gui_num_bytes_read])
                        .iter()
                        .step_by(2)
                    {
                        scope.write_sample(*sample);
                    }
                }

                let points: Vec<Pos2> = scope
                    .into_iter()
                    .enumerate()
                    .map(|(i, sample)| to_screen * pos2(i as f32 / (48000.0 * 1.0), sample))
                    .collect();

                shapes.push(crate::egui::epaint::Shape::line(
                    points,
                    crate::egui::epaint::Stroke::new(1.0, color),
                ));
            }

            ui.painter().extend(shapes);
        });
    }
}
