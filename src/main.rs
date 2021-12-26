use eframe::{egui, epi};

mod stuff;
use crate::stuff::player::Player;
use crate::stuff::playlist::{Playlist, Track};

struct AppState {
    pub player: Player,
    pub playlists: Vec<Playlist>,
    pub current_playlist_idx: Option<usize>,
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
                ui.horizontal(|ui| {
                    for (idx, playlist) in self.playlists.iter().enumerate() {
                        let playlist_tab = ui.button(playlist.get_name().unwrap());

                        if playlist_tab.clicked() {
                            self.current_playlist_idx = Some(idx);
                        }
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
                            tracing::info!("Double clicked {:?}", &track.path);
                            self.player.selected_track = Some(track.clone());
                            self.player.play();
                        }

                        if track_item.clicked() {
                            tracing::info!("Clicked {:?}", &track.path);
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
                let mut new_playlist = Playlist::new();
                new_playlist.set_name("New Playlist".to_string());

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
    };

    let mut window_options = eframe::NativeOptions::default();
    window_options.initial_window_size = Some(egui::Vec2::new(1024., 768.));
    eframe::run_native(Box::new(app_state), window_options);
}
