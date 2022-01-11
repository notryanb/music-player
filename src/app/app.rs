use eframe::{egui, epi};
use itertools::Itertools;

use super::App;
use crate::app::Library;
use crate::app::LibraryItem;
use crate::app::Playlist;
use id3::Tag;
use rayon::prelude::*;
use walkdir::WalkDir;

impl epi::App for App {
    fn on_exit(&mut self) {
        tracing::info!("exiting and saving");
        self.save_state();
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
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

                    if let Some(_selected_track) = &self.player.as_mut().unwrap().selected_track {
                        if play_btn.clicked() {
                            self.player.as_mut().unwrap().play();
                        }

                        if stop_btn.clicked() {
                            self.player.as_mut().unwrap().stop();
                        }

                        if pause_btn.clicked() {
                            self.player.as_mut().unwrap().pause();
                        }

                        if next_btn.clicked() {
                            self.player
                                .as_mut()
                                .unwrap()
                                .next(&self.playlists[(self.current_playlist_idx).unwrap()])
                        }

                        if prev_btn.clicked() {
                            self.player
                                .as_mut()
                                .unwrap()
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
                if self.player.as_ref().unwrap().is_stopped() {
                    ui.label("Stopped");
                } else {
                    if let Some(selected_track) = &self.player.as_ref().unwrap().selected_track {
                        ui.monospace(egui::RichText::new(
                            self.player.as_ref().unwrap().track_state.to_string(),
                        ));

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
                                self.library = Some(Library::new(lib_path.clone()));

                                let tx = self.library_sender.as_ref().unwrap().clone();
                                std::thread::spawn(move || {
                                    let files: Vec<walkdir::DirEntry> = WalkDir::new(&lib_path)
                                        .into_iter()
                                        .filter_map(|e| e.ok())
                                        .skip(1)
                                        .filter(|entry| entry.file_type().is_file())
                                        .collect();

                                    let items = files
                                        .par_iter()
                                        .map(|entry| {
                                            let tag = Tag::read_from_path(&entry.path());

                                            let library_item = match tag {
                                                Ok(tag) => {
                                                    LibraryItem::new(entry.path().to_path_buf())
                                                        .set_title(tag.title())
                                                        .set_artist(tag.artist())
                                                        .set_album(tag.album())
                                                        .set_year(tag.year())
                                                        .set_genre(tag.genre())
                                                        .set_track_number(tag.track())
                                                }
                                                Err(_err) => {
                                                    tracing::warn!(
                                                        "Couldn't parse to id3: {:?}",
                                                        &entry.path()
                                                    );
                                                    LibraryItem::new(entry.path().to_path_buf())
                                                }
                                            };

                                            return library_item;
                                        })
                                        .collect::<Vec<LibraryItem>>();

                                    tx.send(items).expect("Failed to send")
                                });
                            }
                        }

                        if let Some(library) = &self.library {
                            let root_path_string = &library
                                .root_path()
                                .clone()
                                .into_os_string()
                                .into_string()
                                .unwrap();

                            egui::CollapsingHeader::new(egui::RichText::new(root_path_string))
                                .default_open(true)
                                .show(ui, |ui| {
                                    let mut library_items_clone = library.items().clone();

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

                                        let library_group =
                                            egui::CollapsingHeader::new(egui::RichText::new(key))
                                                .default_open(false)
                                                .selectable(true)
                                                .show(ui, |ui| {
                                                    for item in &items {
                                                        let item_label = ui.add(
                                                            egui::Label::new(egui::RichText::new(
                                                                item.title()
                                                                    .unwrap_or("?".to_string()),
                                                            ))
                                                            .sense(egui::Sense::click()),
                                                        );

                                                        if item_label.double_clicked() {
                                                            if let Some(current_playlist_idx) =
                                                                &self.current_playlist_idx
                                                            {
                                                                let current_playlist = &mut self
                                                                    .playlists
                                                                    [*current_playlist_idx];

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
                    });
                });
        });

        self.main_window(ctx);
    }

    fn name(&self) -> &str {
        "Music Player"
    }
}
