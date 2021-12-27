use eframe::{egui, epi};

mod stuff;
use crate::stuff::player::Player;
use crate::stuff::playlist::{Playlist, Track};

struct AppState {
    pub player: Player,
    pub playlists: Vec<Playlist>,
    pub current_playlist_idx: Option<usize>,
    pub playlist_idx_to_remove: Option<usize>,
}

impl epi::App for AppState {
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        egui::TopBottomPanel::top("MusicPlayer").show(ctx, |ui| {
            ui.label("Welcome to MusicPlayer!");
        });

        egui::TopBottomPanel::top("Player stuff").show(ctx, |ui| {
            self.player_ui(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::SidePanel::left("Library Window")
                .min_width(200.)
                .show(ctx, |ui| {
                    ui.label("music library");
                })
        });

        self.main_window(ctx);
    }

    fn name(&self) -> &str {
        "Music Player"
    }
}

impl AppState {
    fn main_window(&mut self, ctx: &egui::CtxRef) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::TopBottomPanel::top("Playlist Tabs").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (idx, playlist) in self.playlists.iter().enumerate() {
                        let playlist_tab = ui.add(
                            egui::Label::new(playlist.get_name().unwrap())
                                .sense(egui::Sense::click()),
                        );

                        if playlist_tab.clicked() {
                            tracing::info!("playlist tab was clicked");
                            self.current_playlist_idx = Some(idx);
                        }

                        if playlist_tab.clicked_by(egui::PointerButton::Secondary) {
                            tracing::info!("Right clicked the playlist tab idx {}", idx);

                            // TODO - make this bring up a context menu, however just delete for
                            // now.

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
                    if ui.button("Add file to playlist").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            tracing::debug!("Adding file to playlist");
                            self.playlists[*current_playlist_idx].add(Track { path });
                        }
                    }

                    for track in self.playlists[*current_playlist_idx].tracks.iter() {
                        let track_item = ui.add(
                            egui::Label::new(track.path.as_path().display())
                                .sense(egui::Sense::click()),
                        );

                        if track_item.double_clicked() {
                            tracing::debug!("Double clicked {:?}", &track.path);
                            self.player.selected_track = Some(track.clone());
                            self.player.play();
                        }

                        if track_item.clicked() {
                            tracing::debug!("Clicked {:?}", &track.path);
                            self.player.selected_track = Some(track.clone());
                        }
                    }
                }
            });
        });
    }

    fn player_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let stop_btn = ui.button("■");
            let play_btn = ui.button("▶");
            let pause_btn = ui.button("⏸");

            if ui.button("Create Playlist +").clicked() {
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

            if let Some(selected_track) = &self.player.selected_track {
                ui.label("Track State: ");
                ui.monospace(&self.player.track_state);

                ui.label(selected_track.path.display());

                if stop_btn.clicked() {
                    self.player.stop();
                }

                if play_btn.clicked() {
                    self.player.play();
                }

                if pause_btn.clicked() {
                    self.player.pause();
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
    };

    let mut window_options = eframe::NativeOptions::default();
    window_options.initial_window_size = Some(egui::Vec2::new(1024., 768.));
    eframe::run_native(Box::new(app_state), window_options);
}
