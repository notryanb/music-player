use eframe::{egui, epi};
use itertools::Itertools;

mod stuff;
use crate::stuff::library::{Library, LibraryItem};
use crate::stuff::player::Player;
use crate::stuff::playlist::Playlist;

struct AppState {
    pub player: Player,
    pub playlists: Vec<Playlist>,
    pub current_playlist_idx: Option<usize>,
    pub playlist_idx_to_remove: Option<usize>,
    pub library: Option<Library>,
}

impl epi::App for AppState {
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
                    ui.button("Play");
                    ui.button("Stop");
                    ui.button("Pause");
                    ui.button("Next");
                    ui.button("Previous");
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
                                .clone()
                                .into_os_string()
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

impl AppState {
    fn main_window(&mut self, ctx: &egui::CtxRef) {
        egui::CentralPanel::default().show(ctx, |_ui| {
            egui::TopBottomPanel::top("Playlist Tabs").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (idx, playlist) in self.playlists.iter().enumerate() {
                        let playlist_tab = ui.add(
                            egui::Label::new(playlist.get_name().unwrap())
                                .sense(egui::Sense::click()),
                        );

                        if playlist_tab.clicked() {
                            self.current_playlist_idx = Some(idx);
                        }

                        // TODO - make this bring up a context menu, however just delete for
                        // now.
                        if playlist_tab.clicked_by(egui::PointerButton::Secondary) {
                            self.playlist_idx_to_remove = Some(idx);
                        }
                    }

                    if let Some(idx) = self.playlist_idx_to_remove {
                        self.playlist_idx_to_remove = None;

                        // Because the current playlist is referenced via index, we need to take
                        // into account that the index may be out of bounds when removing a
                        // playlist. This should be resolved when I figure out how to reference the
                        // actual selected playlist.
                        if let Some(mut current_playlist_idx) = self.current_playlist_idx {
                            if current_playlist_idx == 0 && idx == 0 {
                                self.current_playlist_idx = None;
                            } else if current_playlist_idx >= idx {
                                current_playlist_idx -= 1;
                                self.current_playlist_idx = Some(current_playlist_idx);
                            }
                        }

                        self.playlists.remove(idx);
                    }
                });
            });

            // Playlist contents
            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(current_playlist_idx) = &mut self.current_playlist_idx {
                    self.playlist_table(ui);
                }
            });
        });
    }

    fn playlist_table(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::both().show(ui, |ui| {
            if let Some(current_playlist_idx) = &mut self.current_playlist_idx {
                egui::Grid::new("playlist")
                    .striped(true)
                    .min_col_width(25.)
                    .show(ui, |ui| {
                        // Header
                        ui.label("Playing");
                        ui.label("#");
                        ui.label("Artist");
                        ui.label("Album");
                        ui.label("Title");
                        ui.label("Genre");
                        ui.end_row();

                        // Rows
                        for track in self.playlists[*current_playlist_idx].tracks.iter() {
                            if let Some(selected_track) = &self.player.selected_track {
                                if selected_track == track {
                                    ui.label("▶".to_string());
                                } else {
                                    ui.label(" ".to_string());
                                }
                            } else {
                                ui.label(" ".to_string());
                            }

                            ui.label("0".to_string());

                            let artist_label = ui.add(egui::Label::new(&track.artist().unwrap_or("?".to_string())).sense(egui::Sense::click()));

                            ui.label(&track.album().unwrap_or("?".to_string()));
                            ui.label(&track.title().unwrap_or("?".to_string()));
                            ui.label(&track.genre().unwrap_or("?".to_string()));

                            // Temporary hack because I don't yet know how to treat an entire Row
                            // as a response
                            if artist_label.double_clicked() {
                                self.player.selected_track = Some(track.clone());
                                self.player.play();
                            }

                            if artist_label.clicked() {
                                self.player.selected_track = Some(track.clone());
                            }

                            ui.end_row();
                        }
                    });
            }
        });
    }

    fn player_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let stop_btn = ui.button("■");
            let play_btn = ui.button("▶");
            let pause_btn = ui.button("⏸");
            let prev_btn = ui.button("|◀");
            let next_btn = ui.button("▶|");

            let mut volume = self.player.volume;
            ui.add(
                egui::Slider::new(&mut volume, (0.0 as f32)..=(5.0 as f32))
                    .logarithmic(false)
                    .show_value(false)
                    .clamp_to_range(true),
            );
            self.player.set_volume(volume);

            if let Some(selected_track) = &self.player.selected_track {

                if stop_btn.clicked() {
                    self.player.stop();
                }

                if play_btn.clicked() {
                    self.player.play();
                }

                if pause_btn.clicked() {
                    self.player.pause();
                }

                if prev_btn.clicked() {
                    self.player
                        .previous(&self.playlists[(self.current_playlist_idx).unwrap()])
                }

                if next_btn.clicked() {
                    self.player
                        .next(&self.playlists[(self.current_playlist_idx).unwrap()])
                }
            }
        });
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("App booting...");

    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();

    let app_state = AppState {
        player: Player::new(sink, stream_handle),
        playlists: Vec::new(),
        current_playlist_idx: None,
        playlist_idx_to_remove: None,
        library: None,
    };

    let mut window_options = eframe::NativeOptions::default();
    window_options.initial_window_size = Some(egui::Vec2::new(1024., 768.));
    eframe::run_native(Box::new(app_state), window_options);
}
