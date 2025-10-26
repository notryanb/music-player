use library::{
    Library, LibraryItem, LibraryItemContainer, LibraryPath, LibraryPathId, LibraryPathStatus,
    LibraryView, ViewType,
};
use player::Player;
use playlist::Playlist;
use scope::Scope;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

use itertools::Itertools;

use id3::{Tag, TagLike};
use rayon::prelude::*;

mod app;
mod components;
mod library;
pub mod player;
mod playlist;
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
    LibraryAddPathId(LibraryPathId),
}


#[derive(Serialize, Deserialize)]
pub struct App {
    pub library: Library,

    pub playlists: Vec<Playlist>,

    pub current_playlist_idx: Option<usize>,

    pub show_oscilloscope: bool,

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
    pub scope: Option<Scope>,

    #[serde(skip_serializing, skip_deserializing)]
    pub temp_buf: Option<Vec<f32>>,

    #[serde(skip_serializing, skip_deserializing)]
    pub quit: bool,

    #[serde(skip_serializing, skip_deserializing)]
    pub lib_config_selections: std::collections::HashSet<LibraryPathId>,

    #[serde(skip_serializing, skip_deserializing)]
    pub is_library_cfg_open: bool,

    #[serde(skip_serializing, skip_deserializing)]
    pub is_processing_ui_change: Option<Arc<AtomicBool>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            library: Library::new(),
            playlists: vec![],
            current_playlist_idx: None,
            show_oscilloscope: false,
            process_gui_samples: Arc::new(AtomicBool::new(false)),
            player: None,
            playlist_idx_to_remove: None,
            ui_tx: None,
            ui_rx: None,
            played_audio_buffer: None,
            scope: Some(Scope::new()),
            temp_buf: Some(vec![0.0f32; 4096]),
            quit: false,
            lib_config_selections: Default::default(),
            is_library_cfg_open: false,
            is_processing_ui_change: None,
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

        std::thread::spawn(move || {
            let files = walkdir::WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .skip(1)
                .filter(|entry| {
                    entry.file_type().is_file()
                        && entry.path().extension().unwrap_or(std::ffi::OsStr::new("")) == "mp3"
                })
                .collect::<Vec<_>>();

            let items = files
                .par_iter()
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
                            tracing::warn!("Couldn't parse to id3: {:?}", &entry.path());
                            LibraryItem::new(entry.path().to_path_buf(), path_id)
                        }
                    };

                    return library_item;
                })
                .collect::<Vec<LibraryItem>>();

            tracing::info!("Done parsing library items");

            // Populate the library
            for item in &items {
                cmd_tx
                    .send(UiCommand::LibraryAddItem((*item).clone()))
                    .expect("failed to send library item")
            }

            // Build the views
            let mut library_view = LibraryView {
                view_type: ViewType::Album,
                containers: Vec::new(),
            };

            // In order for group by to work from itertools, items must be consecutive, so sort them first.
            let mut library_items_clone = items.clone();
            library_items_clone.sort_by_key(|item| item.album());

            let grouped_library_by_album = &library_items_clone
                .into_iter()
                .group_by(|item| item.album().unwrap_or("<?>".to_string()).to_string());

            for (album_name, album_library_items) in grouped_library_by_album {
                let lib_item_container = LibraryItemContainer {
                    name: album_name.clone(),
                    items: album_library_items
                        .map(|item| item.clone())
                        .collect::<Vec<LibraryItem>>(),
                };

                library_view.containers.push(lib_item_container.clone());
            }

            cmd_tx
                .send(UiCommand::LibraryAddView(library_view))
                .expect("Failed to send library view");

            cmd_tx
                .send(UiCommand::LibraryAddPathId(path_id))
                .expect("Failed to send library view");
            //lib_path.set_imported();
        });
    }
}
