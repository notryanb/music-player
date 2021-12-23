use std::io::BufReader;

use eframe::{egui, epi};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

mod playlist;
use playlist::{Playlist, Track};

struct AppState {
    pub track_state: TrackState,
    pub selected_file: Option<String>,
    pub playlists: Vec<Playlist>,
    pub current_playlist: Option<Playlist>,
    pub sink: Sink,
    pub stream_handle: OutputStreamHandle,
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
                    for playlist in &self.playlists {
                        let playlist_tab = ui.button(playlist.get_name().unwrap());

                        if playlist_tab.clicked() {
                            self.current_playlist = Some((*playlist).clone());
                        }
                    }
                });
            });

            // Playlist contents
            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(current_playlist) = &mut self.current_playlist {
                    if ui.button("Add file to playlist").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            tracing::info!("Adding file to playlist");
                            current_playlist.add(Track { path });
                        }
                    }

                    for track in current_playlist.tracks.iter() {
                        let track_item = ui.add(
                            egui::Label::new(track.path.as_path().display())
                                .sense(egui::Sense::click()),
                        );

                        if track_item.clicked() {
                            tracing::info!("Clicked {:?}", &track.path);
                            let path_string =
                                track.path.clone().into_os_string().into_string().unwrap();
                            self.selected_file = Some(path_string.clone());
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
                self.current_playlist = Some(new_playlist.clone());
            }
            
            if let Some(selected_file) = &self.selected_file {
                ui.label("Track State: ");
                ui.monospace(&self.track_state);
                
                ui.label(selected_file);

                let file = BufReader::new(std::fs::File::open(&selected_file).expect("Failed to open file"));
                let source = Decoder::new(file).expect("Failed to decode audio file");   

                if stop_btn.clicked() {
                    match &self.track_state {
                        TrackState::Playing | TrackState::Paused => {
                            self.track_state = TrackState::Stopped;
                            tracing::info!("Stopping the source that is in the sink's queue");
                            self.sink.stop();
        
                            if self.sink.empty() {
                                tracing::info!("STOPPED: The sink has no more sounds to play");
                            }
                        },
                        _ => ()
                    }
                }
    
                if play_btn.clicked() {
                    match self.track_state {
                        TrackState::Unstarted | TrackState::Stopped => {
                            self.track_state = TrackState::Playing;
                            tracing::info!("Appending audio source to sink queue");
    
                            if self.sink.empty() {
                                tracing::info!("Playing: The sink has no more sounds to play");
                            }
                            self.sink = Sink::try_new(&self.stream_handle).unwrap();
                            
                            self.sink.append(source);
    
                            if self.sink.empty() {
                                tracing::warn!("Playing: uh, we just appended a source. This should NOT be hit");
                            }
                        },
                        TrackState::Paused => {
                            // TODO! - Add check if the sink has a source in it's queue.
                            tracing::info!("Should already have source in the sink queue, going to play");
                            self.sink.play();
                            self.track_state = TrackState::Playing;
                        },
                        _ => ()
                    }
                }
    
                if pause_btn.clicked() {
                    match self.track_state {
                        TrackState::Playing => {
                            self.track_state = TrackState::Paused;
                            self.sink.pause();
                        },
                        _ => ()
                    }
                }
            }
        });
    }
}

pub enum TrackState {
    Unstarted,
    Stopped,
    Playing,
    Paused,
}

impl std::fmt::Display for TrackState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TrackState::Unstarted => write!(f, "Unstarted"),
            TrackState::Stopped => write!(f, "Stopped"),
            TrackState::Playing => write!(f, "Playing"),
            TrackState::Paused => write!(f, "Paused"),
        }
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("App booting...");

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let app_state = AppState {
        track_state: TrackState::Unstarted,
        selected_file: None,
        sink,
        stream_handle,
        playlists: Vec::new(),
        current_playlist: None,
    };

    let mut window_options = eframe::NativeOptions::default();
    window_options.initial_window_size = Some(egui::Vec2::new(1024., 768.));
    eframe::run_native(Box::new(app_state), window_options);
}
