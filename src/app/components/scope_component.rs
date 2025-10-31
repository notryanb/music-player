use super::AppComponent;
use crate::app::App;
use crate::egui::epaint::*;
use crate::egui::{pos2, vec2, Frame, Pos2, Rect};
use rb::RbConsumer;

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
                if let Some(audio_buf) = &ctx.played_audio_buffer {
                    if let Some(local_buf) = &mut ctx.temp_buf {
                        let num_bytes_read = audio_buf.read(&mut local_buf[..]).unwrap_or(0);

                        // TERRIBLE RMS METER EXPERIMENT
                        // This is terrible, but I just wanted to see if it works.
                        // The main app state should hold the buffer and then the individual components
                        // should be able to get their own copies to use. The scope kinda owns this logic
                        // when it definitely shouldn't
                        let left_samples_sum = local_buf
                            .iter()
                            .skip(0)
                            .step_by(2)
                            .copied()
                            .map(|x| x * x)
                            .sum::<f32>();

                        let left_rms = (left_samples_sum / (local_buf.len() as f32 / 2.0)).sqrt();
                        let left_rms_db = 20.0 * left_rms.log10();

                        let right_samples_sum = local_buf
                            .iter()
                            .skip(1)
                            .step_by(2)
                            .copied()
                            .map(|x| x * x)
                            .sum::<f32>();

                        let right_rms = (right_samples_sum / (local_buf.len() as f32 / 2.0)).sqrt();
                        let right_rms_db = 20.0 * right_rms.log10();
                        ctx.rms_meter = [left_rms_db, right_rms_db];
                        // END terrible experiment

                        if num_bytes_read > 0 {

                            for sample in (local_buf[0..num_bytes_read]).iter().step_by(2) {
                                scope.write_sample(*sample);
                            }
                        }
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
