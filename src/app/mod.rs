use library::{Library, LibraryItem};
use player::Player;
use playlist::Playlist;
use serde::{Serialize, Deserialize};
use serde_json;
use std::{fs::File, io::Read, io::Write};

use eframe::{egui};

mod app;
mod library;
pub mod player;
mod playlist;

#[derive(Serialize, Deserialize)]
pub struct App {
    pub library: Option<Library>,

    pub playlists: Vec<Playlist>,

    pub current_playlist_idx: Option<usize>,
    
    #[serde(skip_serializing, skip_deserializing)]
    pub player: Option<Player>,

    #[serde(skip_serializing, skip_deserializing)]
    pub playlist_idx_to_remove: Option<usize>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            library: None,
            playlists: vec![],
            current_playlist_idx: None,
            player: None,
            playlist_idx_to_remove: None,
        }
    }
}

impl App {
    pub fn load() -> Self {
        let mut saved_state = String::new();
        let mut file = File::open("./music_player_app.json").unwrap();
        file.read_to_string(&mut saved_state).unwrap();

        let app = serde_json::from_str(&saved_state).unwrap();
        app
    }

    pub fn save_state(&self) {
        let config = serde_json::to_string(&self).unwrap();
        let location = "./music_player_app.json";

        let mut file = File::create(location).unwrap();
        file.write_all(config.as_bytes()).unwrap();
    }

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
                if let Some(_current_playlist_idx) = &mut self.current_playlist_idx {
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
                            if let Some(selected_track) = &self.player.as_ref().unwrap().selected_track {
                                if selected_track == track {
                                    ui.label("▶".to_string());
                                } else {
                                    ui.label(" ".to_string());
                                }
                            } else {
                                ui.label(" ".to_string());
                            }

                            if let Some(track_number) = &track.track_number() {
                                ui.label(&track_number.to_string());
                            } else {
                                ui.label("");
                            }

                            let artist_label = ui.add(
                                egui::Label::new(&track.artist().unwrap_or("?".to_string()))
                                    .sense(egui::Sense::click()),
                            );

                            ui.label(&track.album().unwrap_or("?".to_string()));
                            ui.label(&track.title().unwrap_or("?".to_string()));
                            ui.label(&track.genre().unwrap_or("?".to_string()));

                            // Temporary hack because I don't yet know how to treat an entire Row
                            // as a response
                            if artist_label.double_clicked() {
                                self.player.as_mut().unwrap().selected_track = Some(track.clone());
                                self.player.as_mut().unwrap().play();
                            }

                            if artist_label.clicked() {
                                self.player.as_mut().unwrap().selected_track = Some(track.clone());
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

            let mut volume = self.player.as_ref().unwrap().volume;
            ui.add(
                egui::Slider::new(&mut volume, (0.0 as f32)..=(5.0 as f32))
                    .logarithmic(false)
                    .show_value(false)
                    .clamp_to_range(true),
            );
            self.player.as_mut().unwrap().set_volume(volume);

            if let Some(_selected_track) = &self.player.as_mut().unwrap().selected_track {
                if stop_btn.clicked() {
                    self.player.as_mut().unwrap().stop();
                }

                if play_btn.clicked() {
                    self.player.as_mut().unwrap().play();
                }

                if pause_btn.clicked() {
                    self.player.as_mut().unwrap().pause();
                }

                if prev_btn.clicked() {
                    self.player.as_mut().unwrap()
                        .previous(&self.playlists[(self.current_playlist_idx).unwrap()])
                }

                if next_btn.clicked() {
                    self.player.as_mut().unwrap()
                        .next(&self.playlists[(self.current_playlist_idx).unwrap()])
                }
            }
        });
    }
}


