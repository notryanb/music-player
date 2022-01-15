use eframe::{egui, epi};

use super::App;
use crate::app::components::{
    footer::Footer, library_component::LibraryComponent, menu_bar::MenuBar,
    player_component::PlayerComponent, AppComponent,
};

impl epi::App for App {
    fn on_exit(&mut self) {
        tracing::info!("exiting and saving");
        self.save_state();
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        if self.quit {
            frame.quit();
        }

        ctx.request_repaint();

        if let Some(rx) = &self.library_receiver {
            match rx.try_recv() {
                Ok(library_items) => {
                    for item in library_items {
                        if let Some(library) = &mut self.library {
                            library.add_item(item);
                        }
                    }
                }
                Err(_) => (),
            }
        }

        if let Some(selected_track) = &self.player.as_mut().unwrap().selected_track {
            let display = format!(
                "{} - {} [ Music Player ]",
                &selected_track.artist().unwrap_or("?".to_string()),
                &selected_track.title().unwrap_or("?".to_string())
            );

            frame.set_window_title(&display);
        }

        egui::TopBottomPanel::top("MusicPlayer").show(ctx, |ui| {
            MenuBar::add(self, ui);
        });

        egui::TopBottomPanel::top("Player").show(ctx, |ui| {
            // self.player_ui(ui);
            PlayerComponent::add(self, ui);
        });

        egui::TopBottomPanel::bottom("Footer").show(ctx, |ui| {
            Footer::add(self, ui);
        });

        egui::CentralPanel::default().show(ctx, |_ui| {
            egui::SidePanel::left("Library Window")
                .default_width(250.0)
                .show(ctx, |ui| {
                    LibraryComponent::add(self, ui);
                });
        });

        self.main_window(ctx);
    }

    fn name(&self) -> &str {
        "Music Player"
    }
}
