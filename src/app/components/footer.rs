use super::AppComponent;
use crate::app::App;

pub struct Footer;

impl AppComponent for Footer {
    type Context = App;

    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui) {
        ui.horizontal(|ui| {
            if ctx.player.as_ref().unwrap().is_stopped() {
                ui.label("Stopped");
            } else {
                if let Some(selected_track) = &ctx.player.as_ref().unwrap().selected_track {
                    ui.monospace(eframe::egui::RichText::new(
                        ctx.player.as_ref().unwrap().track_state.to_string(),
                    ));

                    ui.label(eframe::egui::RichText::new(
                        &selected_track
                            .path()
                            .as_path()
                            .file_name()
                            .unwrap()
                            .to_os_string()
                            .into_string()
                            .unwrap(),
                    ));
                }
            }
        });
    }
}
