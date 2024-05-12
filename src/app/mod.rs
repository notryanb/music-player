use library::{Library, LibraryItem, LibraryView};
use player::Player;
use playlist::Playlist;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};

mod app;
mod components;
mod library;
pub mod player;
mod playlist;
mod output;
mod resampler;

use crate::app::output::AudioOutput;


pub enum AudioCommand {
    Stop,
    Play,
    Pause,
    Seek(u32), // Maybe this should represent a duration?
    LoadFile(std::path::PathBuf),
    Select(usize),
    SetVolume(f32),
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

    // fn process_audio_cmds(&mut self) {
    //         match audio_rx.try_recv() {
    //             Ok(cmd) => {
    //                 match cmd {
    //                     AudioCommand::Seek(seconds) => {
    //                         tracing::info!("Processing SEEK command for {} seconds", seconds);


    //                         // // TODO - Need to figure out how to implement using iterator + ring buf
    //                         // let guard = current_track_sample_rate.lock().unwrap();
    //                         // let sample_num = *guard * seconds as u32;
    //                         // drop(guard);
    //                         // cursor.swap(sample_num, Relaxed);
    //                     }
    //                     AudioCommand::Stop => {
    //                         tracing::info!("Processing STOP command");
    //                         {
    //                             let mut state_guard = state.lock().unwrap();
    //                             *state_guard = PlayerState::Stopped;
    //                         }

    //                         // // TODO - Doing this incurs an extra allocation.
    //                         // // Also, when stopping the ring buffer isn't flushed, so when
    //                         // // you start to play again there is still data to consume from
    //                         // // the previous track
    //                         // audio_source = audio_source_clone;
    //                     }
    //                     AudioCommand::Pause => {
    //                         tracing::info!("Processing PAUSE command");


    //                         // let mut state_guard = state.lock().unwrap();
    //                         // *state_guard = PlayerState::Paused;
    //                     }
    //                     AudioCommand::Play => {
    //                         tracing::info!("Processing PLAY command");
                            

    //                         // let mut state_guard = state.lock().unwrap();
    //                         // *state_guard = PlayerState::Playing;
    //                     }
    //                     AudioCommand::LoadFile(path) => {
    //                         // let resampled_audio = load_file(path, device_sample_rate);
    //                         // audio_source = Some(resampled_audio);
    //                         tracing::info!("Processing LOAD FILE command for path: {:?}", &path);

    //                     }
    //                     _ => tracing::warn!("Unhandled case in audio command loop"),
    //                 }
    //             }
    //             Err(_) => (), // When no commands are sent, this will evaluate. aka - it is the
    //                             // common case. No need to print anything
    //         }
    // }
}
