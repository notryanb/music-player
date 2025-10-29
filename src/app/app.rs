use eframe::egui;
use std::sync::atomic::Ordering;

use super::{App, UiCommand};
use crate::app::components::{
    footer::Footer, library_component::LibraryComponent, menu_bar::MenuBar,
    player_component::PlayerComponent, playlist_table::PlaylistTable, playlist_tabs::PlaylistTabs,
    scope_component::ScopeComponent, AppComponent,
};

use crate::meter::{Meter, DB_TICKS, DB_SECTIONS, DbMapper};

impl eframe::App for App {
    fn on_exit(&mut self, _ctx: Option<&eframe::glow::Context>) {
        tracing::info!("exiting and saving");
        self.save_state();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        ctx.request_repaint();

        // Main event processing loop
        if let Some(cmd_rx) = &self.ui_rx {
            match cmd_rx.try_recv() {
                Ok(cmd) => match cmd {
                    UiCommand::LibraryAddItem(lib_item) => self.library.add_item(lib_item),
                    UiCommand::LibraryAddView(lib_view) => self.library.add_view(lib_view),
                    UiCommand::LibraryAddPathId(path_id) => {
                        self.library.set_path_to_imported(path_id)
                    },
                    UiCommand::CurrentTimestamp(seek_timestamp) => {
                        self.player.as_mut().unwrap().set_seek_to_timestamp(seek_timestamp);
                    }
                    UiCommand::TotalTrackDuration(dur) => {
                        tracing::info!("Received Duration: {}", dur);
                        self.player.as_mut().unwrap().set_duration(dur);
                    }
                    UiCommand::SampleRate(sr) => {
                        tracing::info!("Received sample_rate: {}", sr);
                        self.player.as_mut().unwrap().set_sample_rate(sr);
                    }
                    UiCommand::AudioFinished => {
                        tracing::info!("Track finished, getting next...");

                        self.player
                            .as_mut()
                            .unwrap()
                            .next(&self.playlists[(self.current_playlist_idx).unwrap()]);
                    },
                },
                Err(_) => (),
            }
        }

        if let Some(selected_track) = &self.player.as_mut().unwrap().selected_track {
            let display = format!(
                "{} - {} [ Music Player ]",
                &selected_track.artist().unwrap_or("?".to_string()),
                &selected_track.title().unwrap_or("?".to_string())
            );

            ctx.send_viewport_cmd(egui::ViewportCommand::Title(display));
        }

        egui::TopBottomPanel::top("MusicPlayer").show(ctx, |ui| {
            MenuBar::add(self, ui);
        });

        egui::TopBottomPanel::top("Player").show(ctx, |ui| {
            egui::Frame::new()
                .inner_margin(6)
                .show(ui, |ui| {
                    PlayerComponent::add(self, ui);
                });

            if self.show_oscilloscope {
                if !self.process_gui_samples.load(Ordering::Relaxed) {
                    self.process_gui_samples.store(true, Ordering::Relaxed);
                }

                eframe::egui::Window::new("Oscilloscope")
                    .default_width(600.0)
                    .default_height(400.0)
                    .resizable([true, true])
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ScopeComponent::add(self, ui);
                    });
            }
        });

        egui::TopBottomPanel::bottom("Footer").show(ctx, |ui| {
            Footer::add(self, ui);
        });

        egui::CentralPanel::default().show(ctx, |_ui| {
            egui::SidePanel::left("Library Window")
                .default_width(500.0)
                .show(ctx, |ui| {
                    LibraryComponent::add(self, ui);
                });
        });

        egui::CentralPanel::default().show(ctx, |_ui| {
            egui::TopBottomPanel::top("Playlist Tabs").show(ctx, |ui| {
                PlaylistTabs::add(self, ui);
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(_current_playlist_idx) = &mut self.current_playlist_idx {
                    egui::ScrollArea::both().show(ui, |ui| {
                        PlaylistTable::add(self, ui);
                    });
                }

                // Temporary
                if self.show_oscilloscope {
                    eframe::egui::Window::new("RMS Meter")
                        .default_width(480.0)
                        .default_height(640.0)
                        .resizable([true, true])
                        .collapsible(false)
                        .show(ctx, |ui| {
                            ui.add(
                                Meter::new(&[self.rms_meter[0], self.rms_meter[1]])
                                .with_ticks(&DB_TICKS)
                                .with_sections(&DB_SECTIONS)
                                .with_text_above("RMS")
                                .show_max(true)
                                .with_mapper(&DbMapper),
                            );
                            
                        });
                }
            });
        });
    }
}
