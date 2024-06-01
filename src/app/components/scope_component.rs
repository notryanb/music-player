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

            let desired_size = ui.available_width() * vec2(1.0, 0.35);
            let (_id, rect) = ui.allocate_space(desired_size);

            let to_screen =
                emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0), rect);
            let mut shapes = vec![];

            // TODO - Need to figure out how to handle moving the data out of the scope buffer option... This creates redundant code.
            if let Some(ref mut data) = &mut ctx.scope_buffer {
                if let Some(audio_buf) = &ctx.played_audio_buffer {
                    // let _num_bytes_read = audio_buf.read(&mut data).unwrap_or(0);
                    let _num_bytes_read = audio_buf.read(data).unwrap_or(0);
                }

                let points: Vec<Pos2> = data
                    .iter()
                    .step_by(2)
                    .enumerate()
                    .map(|(i, sample)| to_screen * pos2(i as f32, *sample))
                    .collect();

                shapes.push(crate::egui::epaint::Shape::line(
                    points,
                    crate::egui::epaint::Stroke::new(2.0, color),
                ));
            } else {
                let mut data = vec![0.0f32; 4096];

                if let Some(audio_buf) = &ctx.played_audio_buffer {
                    // let _num_bytes_read = audio_buf.read(&mut data).unwrap_or(0);
                    let _num_bytes_read = audio_buf.read(&mut data).unwrap_or(0);
                }

                let points: Vec<Pos2> = data
                    .iter()
                    .step_by(2)
                    .enumerate()
                    .map(|(i, sample)| to_screen * pos2(i as f32, *sample))
                    .collect();

                shapes.push(crate::egui::epaint::Shape::line(
                    points,
                    crate::egui::epaint::Stroke::new(2.0, color),
                ));
            }

            // EGUI Dancing Strings Example
            // for &mode in &[2,3,5] {
            //     let mode = mode as f64;
            //     let n = 120;
            //     let speed = 1.5;

            //     let points: Vec<Pos2> = (0..=n)
            //         .map(|i| {
            //             let t = i as f64 / (n as f64);
            //             let amp = (time * speed * mode).sin() / mode;
            //             let y = amp * (t * std::f64::consts::TAU / 2.0 * mode).sin();
            //             to_screen * pos2(t as f32, y as f32)
            //         })
            //         .collect();

            //     let thickness = 10.0 / mode as f32;
            //     shapes.push(crate::egui::epaint::Shape::line(points, crate::egui::epaint::Stroke::new(thickness, color)));
            // }

            ui.painter().extend(shapes);
        });
    }
}
