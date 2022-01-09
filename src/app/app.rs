use eframe::{egui, epi};
use itertools::Itertools;

use super::App;
use crate::app::Playlist;
use crate::app::Library;
use crate::app::LibraryItem;

impl epi::App for App {
    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        if let Some(selected_track) = &self.player.selected_track {
            let display = format!(
                "{} - {} [ Music Player ]",
                &selected_track.artist().unwrap_or("?".to_string()),
                &selected_track.title().unwrap_or("?".to_string())
            );

            frame.set_window_title(&display);
        }

        egui::TopBottomPanel::top("MusicPlayer").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.button("Open");

                    ui.separator();

                    ui.button("Add Files");
                    ui.button("Add Folders");

                    ui.separator();

                    if ui.button("New Playlist").clicked() {
                        let default_name_count = self
                            .playlists
                            .iter()
                            .filter(|pl| pl.get_name().unwrap().starts_with("New Playlist"))
                            .count();
                        let playlist_name = match default_name_count {
                            0 => "New Playlist".to_string(),
                            _ => format!("New Playlist ({})", default_name_count - 1),
                        };

                        let mut new_playlist = Playlist::new();
                        new_playlist.set_name(playlist_name);

                        self.playlists.push(new_playlist.clone());
                        self.current_playlist_idx = Some(self.playlists.len() - 1);
                    }
                    ui.button("Load Playlist");
                    ui.button("Save Playlist");

                    ui.separator();

                    ui.button("Preferences");

                    ui.separator();

                    if ui.button("Exit").clicked() {
                        frame.quit();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    ui.button("Remove duplicates");
                });

                ui.menu_button("Playback", |ui| {
                    let play_btn = ui.button("Play");
                    let stop_btn = ui.button("Stop");
                    let pause_btn = ui.button("Pause");
                    let next_btn = ui.button("Next");
                    let prev_btn = ui.button("Previous");

                    if let Some(selected_track) = &self.player.selected_track {
                        if play_btn.clicked() {
                            self.player.play();
                        }

                        if stop_btn.clicked() {
                            self.player.stop();
                        }

                        if pause_btn.clicked() {
                            self.player.pause();
                        }

                        if next_btn.clicked() {
                            self.player
                                .next(&self.playlists[(self.current_playlist_idx).unwrap()])
                        }

                        if prev_btn.clicked() {
                            self.player
                                .previous(&self.playlists[(self.current_playlist_idx).unwrap()])
                        }
                    }
                });

                ui.menu_button("Library", |ui| {
                    ui.button("Configure");
                });

                ui.menu_button("Help", |ui| {
                    ui.button("About");
                });
            });
        });

        egui::TopBottomPanel::top("Player stuff").show(ctx, |ui| {
            self.player_ui(ui);
        });

        egui::TopBottomPanel::bottom("Footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.player.is_stopped() {
                    ui.label("Stopped");
                } else {
                    if let Some(selected_track) = &self.player.selected_track {
                        ui.monospace(egui::RichText::new(self.player.track_state.to_string()));

                        ui.label(egui::RichText::new(
                            &selected_track
                                .path()
                                .as_path()
                                .file_name()
                                .unwrap()
                                .clone()
                                .to_os_string()
                                .into_string()
                                .unwrap(),
                        ));
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |_ui| {
            egui::SidePanel::left("Library Window")
                .default_width(250.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::both().show(ui, |ui| {
                        if ui.button("Add Library path").clicked() {
                            if let Some(lib_path) = rfd::FileDialog::new().pick_folder() {
                                tracing::info!("adding library path...");
                                let mut library = Library::new(lib_path);
                                library.build();
                                self.library = Some(library);
                            }
                        }

                        if let Some(library) = &self.library {
                            if let Some(library_items) = &library.items() {
                                let root_path_string = &library
                                    .root_path()
                                    .clone()
                                    .into_os_string()
                                    .into_string()
                                    .unwrap();

                                egui::CollapsingHeader::new(egui::RichText::new(root_path_string))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        let mut library_items_clone = library_items.clone();

                                        // In order for group by to work from itertools, items must be consecutive, so sort them first.
                                        library_items_clone.sort_by_key(|item| item.album());

                                        for (key, group) in
                                            &library_items_clone.into_iter().group_by(|item| {
                                                item.album().unwrap_or("?".to_string()).to_string()
                                            })
                                        {
                                            let items = group
                                                .map(|item| item.clone())
                                                .collect::<Vec<LibraryItem>>();

                                            let library_group = egui::CollapsingHeader::new(
                                                egui::RichText::new(key),
                                            )
                                            .default_open(false)
                                            .selectable(true)
                                            .show(ui, |ui| {
                                                for item in &items {
                                                    let item_label = ui.add(
                                                        egui::Label::new(egui::RichText::new(
                                                            item.title().unwrap_or("?".to_string()),
                                                        ))
                                                        .sense(egui::Sense::click()),
                                                    );

                                                    if item_label.double_clicked() {
                                                        if let Some(current_playlist_idx) =
                                                            &self.current_playlist_idx
                                                        {
                                                            let current_playlist = &mut self
                                                                .playlists[*current_playlist_idx];

                                                            current_playlist.add(item.clone());
                                                        }
                                                    }
                                                }
                                            });

                                            if let Some(current_playlist_idx) =
                                                &self.current_playlist_idx
                                            {
                                                let current_playlist =
                                                    &mut self.playlists[*current_playlist_idx];

                                                if library_group.header_response.double_clicked() {
                                                    for item in items {
                                                        current_playlist.add(item.clone());
                                                    }
                                                }
                                            }
                                        }
                                    });
                            }
                        }
                    });
                });
        });

        self.main_window(ctx);
    }

    fn name(&self) -> &str {
        "Music Player"
    }
}
