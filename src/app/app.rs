use eframe::{egui, epi};
use itertools::Itertools;

use super::App;
use crate::app::components::{footer::Footer, menu_bar::MenuBar, AppComponent};
use crate::app::Library;
use crate::app::LibraryItem;

use id3::Tag;
use rayon::prelude::*;
use walkdir::WalkDir;

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

        egui::TopBottomPanel::top("Player stuff").show(ctx, |ui| {
            self.player_ui(ui);
        });

        egui::TopBottomPanel::bottom("Footer").show(ctx, |ui| {
            Footer::add(self, ui);
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
