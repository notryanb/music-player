use library::{Library, LibraryItem, LibraryView};
use player::Player;
use playlist::Playlist;
use scope::Scope;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};

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
}

#[derive(Serialize, Deserialize)]
pub struct App {
    pub library: Option<Library>,

    pub playlists: Vec<Playlist>,

    pub current_playlist_idx: Option<usize>,

    #[serde(skip_serializing, skip_deserializing)]
    pub player: Option<Player>,

    #[serde(skip_serializing, skip_deserializing)]
    pub playlist_idx_to_remove: Option<usize>,

    #[serde(skip_serializing, skip_deserializing)]
    pub library_sender: Option<Sender<Library>>,

    #[serde(skip_serializing, skip_deserializing)]
    pub library_receiver: Option<Receiver<Library>>,

    #[serde(skip_serializing, skip_deserializing)]
    pub played_audio_buffer: Option<rb::Consumer<f32>>,

    #[serde(skip_serializing, skip_deserializing)]
    pub scope: Option<Scope>,
    
    #[serde(skip_serializing, skip_deserializing)]
    pub temp_buf: Option<Vec<f32>>,

    #[serde(skip_serializing, skip_deserializing)]
    pub quit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            library: None,
            playlists: vec![],
            current_playlist_idx: None,
            player: None,
            playlist_idx_to_remove: None,
            library_sender: None,
            library_receiver: None,
            played_audio_buffer: None,
            scope: Some(Scope::new()),
            temp_buf: Some(vec![0.0f32; 4096]),
            quit: false,
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
}
