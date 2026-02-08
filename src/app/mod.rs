use library::{
    Library, LibraryItem, LibraryItemContainer, LibraryPath, LibraryPathId, LibraryPathStatus,
    LibraryView, ViewType,
};
use player::Player;
use playlist::Playlist;
use rms_calculator::RmsCalculator;
use scope::Scope;

use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};

use id3::{Tag, TagLike};
use rayon::prelude::*;
use rayon::ThreadPool;

mod app;
mod components;
mod library;
pub mod meter;
pub mod player;
mod playlist;
pub mod rms_calculator;
pub mod scope;

pub enum AudioCommand {
    Stop,
    Play,
    Pause,
    Seek(u64),
    LoadFile(std::path::PathBuf),
    Select(usize),
    SetVolume(f32),
}

pub enum UiCommand {
    AudioFinished,
    TotalTrackDuration(u64),
    CurrentTimestamp(u64),
    SampleRate(f32),
    LibraryAddView(LibraryView),
    LibraryAddItem(LibraryItem),
    LibraryAddItems(Vec<LibraryItem>),
    LibraryAddPathId(LibraryPathId),
}

#[derive(Serialize, Deserialize)]
pub struct App {
    pub library: Library,

    pub playlists: Vec<Playlist>,

    pub current_playlist_idx: Option<usize>,

    pub show_oscilloscope: bool,

    pub rms_meter_window_size_millis: u16,

    pub device_sample_rate: f32,

    #[serde(skip_serializing, skip_deserializing)]
    pub rms_calc_left: RmsCalculator,

    #[serde(skip_serializing, skip_deserializing)]
    pub rms_calc_right: RmsCalculator,

    #[serde(skip_serializing, skip_deserializing)]
    pub show_preferences_window: bool,

    #[serde(skip_serializing, skip_deserializing)]
    pub process_gui_samples: Arc<AtomicBool>,

    #[serde(skip_serializing, skip_deserializing)]
    pub player: Option<Player>,

    #[serde(skip_serializing, skip_deserializing)]
    pub playlist_idx_to_remove: Option<usize>,

    #[serde(skip_serializing, skip_deserializing)]
    pub ui_rx: Option<Receiver<UiCommand>>,

    #[serde(skip_serializing, skip_deserializing)]
    pub ui_tx: Option<Sender<UiCommand>>,

    #[serde(skip_serializing, skip_deserializing)]
    pub played_audio_buffer: Option<rb::Consumer<f32>>,

    #[serde(skip_serializing, skip_deserializing)]
    pub ui_audio_buffer: Vec<f32>,

    #[serde(skip_serializing, skip_deserializing)]
    pub gui_num_bytes_read: usize,

    #[serde(skip_serializing, skip_deserializing)]
    pub scope: Option<Scope>,

    #[serde(skip_serializing, skip_deserializing)]
    pub rms_meter: [f32; 2],

    #[serde(skip_serializing, skip_deserializing)]
    pub quit: bool,

    #[serde(skip_serializing, skip_deserializing)]
    pub lib_config_selections: std::collections::HashSet<LibraryPathId>,

    #[serde(skip_serializing, skip_deserializing)]
    pub is_library_cfg_open: bool,

    #[serde(skip_serializing, skip_deserializing)]
    pub is_editing_playlist_name: bool,

    #[serde(skip_serializing, skip_deserializing)]
    pub is_processing_ui_change: Option<Arc<AtomicBool>>,

    #[serde(skip_serializing, skip_deserializing)]
    pub thread_pool: Option<Arc<ThreadPool>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            library: Library::new(),
            playlists: vec![],
            current_playlist_idx: None,
            show_oscilloscope: false,
            show_preferences_window: false,
            device_sample_rate: 44100.0,
            rms_meter_window_size_millis: 250,
            rms_calc_left: RmsCalculator::new(5000),
            rms_calc_right: RmsCalculator::new(5000),
            process_gui_samples: Arc::new(AtomicBool::new(false)),
            player: None,
            playlist_idx_to_remove: None,
            ui_tx: None,
            ui_rx: None,
            played_audio_buffer: None,
            ui_audio_buffer: vec![0.0f32; 4096],
            gui_num_bytes_read: 0,
            scope: Some(Scope::new()),
            rms_meter: [f32::NEG_INFINITY, f32::NEG_INFINITY],
            quit: false,
            lib_config_selections: Default::default(),
            is_library_cfg_open: false,
            is_editing_playlist_name: false,
            is_processing_ui_change: None,
            thread_pool: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TempError {
    MissingAppState,
}

impl std::fmt::Display for TempError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Couldn't load app state")
    }
}

impl App {
    pub fn load() -> Result<Self, TempError> {
        confy::load("music_player", None).map_err(|_| TempError::MissingAppState)
    }

    pub fn save_state(&self) {
        let store_result = confy::store("music_player", None, &self);
        match store_result {
            Ok(_) => tracing::info!("Store was successfull"),
            Err(err) => tracing::error!("Failed to store the app state: {}", err),
        }
    }

    pub fn quit(&mut self) {
        self.quit = true;
    }

    // Spawns a background thread and imports files
    // from each unimported library path
    // TODO - Time and profile this thread
    fn import_library_paths(&self, lib_path: &LibraryPath) {
        if lib_path.status() == LibraryPathStatus::Imported {
            tracing::info!("already imported library path...");
            return;
        }

        tracing::info!("adding library path...");

        let cmd_tx = self.ui_tx.as_ref().unwrap().clone();
        let path = lib_path.path().clone();
        let path_id = lib_path.id().clone();

        if let Some(thread_pool) = &self.thread_pool {
            let thread_pool = thread_pool.clone();

            std::thread::spawn(move || {
                let tx = Mutex::new(cmd_tx.clone());

                let items: Vec<LibraryItem> = thread_pool.install(|| {
                    walkdir::WalkDir::new(path)
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .skip(1)
                        .filter(|entry| {
                            entry.file_type().is_file()
                                && entry.path().extension().unwrap_or(std::ffi::OsStr::new("")) == "mp3"
                        })
                        .par_bridge()
                        .map(|entry| {
                            let tag = Tag::read_from_path(&entry.path());

                            let library_item = match tag {
                                Ok(tag) => LibraryItem::new(entry.path().to_path_buf(), path_id)
                                    .set_title(tag.title())
                                    .set_artist(tag.artist())
                                    .set_album(tag.album())
                                    .set_year(tag.year())
                                    .set_genre(tag.genre())
                                    .set_track_number(tag.track()),
                                Err(_err) => {
                                    // tracing::warn!("Couldn't parse to id3: {:?}", &entry.path());
                                    LibraryItem::new(entry.path().to_path_buf(), path_id)
                                }
                            };

                            return library_item;
                        })
                        .inspect(|item| {
                            tx
                                .lock()
                                .unwrap()
                                .send(UiCommand::LibraryAddItem(item.clone()))
                                .expect("Failed to send Library items");
                         })
                        .collect::<Vec<LibraryItem>>()
                });

                tracing::info!("Completed adding path to library");

                let mut grouped: BTreeMap<String, Vec<&LibraryItem>> = BTreeMap::new();
                for item in &items {
                    let key = item.album().unwrap_or_else(|| "<?>".to_string());
                    grouped.entry(key).or_default().push(item);
                }

                let library_view = LibraryView {
                    view_type: ViewType::Album,
                    containers: grouped
                        .into_iter()
                        .map(|(name, items)| LibraryItemContainer {
                            name,
                            items: items.into_iter().cloned().collect(),
                        })
                        .collect(),
                };

                cmd_tx
                    .send(UiCommand::LibraryAddView(library_view))
                    .expect("Failed to send library view");

                cmd_tx
                    .send(UiCommand::LibraryAddPathId(path_id))
                    .expect("Failed to send library view");

                //lib_path.set_imported();
                tracing::info!("Completed creating library view");
            });
        }
    }
}
