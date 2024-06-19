use crate::app::library::LibraryItem;
use crate::app::playlist::Playlist;
use crate::{AudioCommand, UiCommand};
use std::sync::atomic::AtomicU32;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

pub struct Player {
    pub track_state: TrackState,
    pub selected_track: Option<LibraryItem>,
    pub audio_tx: Sender<AudioCommand>,
    pub ui_rx: Receiver<UiCommand>,
    pub volume: f32,
    pub seek_to_timestamp: u64,
    pub duration: u64,
    pub cursor: Arc<AtomicU32>, // This can "overflow"
}

impl Player {
    pub fn new(
        audio_cmd_tx: Sender<AudioCommand>,
        ui_cmd_rx: Receiver<UiCommand>,
        cursor: Arc<AtomicU32>,
    ) -> Self {
        Self {
            track_state: TrackState::Unstarted,
            selected_track: None,
            audio_tx: audio_cmd_tx,
            ui_rx: ui_cmd_rx,
            volume: 1.0,
            seek_to_timestamp: 0, // TODO: This should have subsecond precision, but is okay for now.
            duration: 0,
            cursor,
        }
    }

    pub fn select_track(&mut self, track: Option<LibraryItem>) {
        self.selected_track = track;

        if let Some(track) = &self.selected_track {
            self.audio_tx
                .send(AudioCommand::LoadFile(track.path()))
                .expect("Failed to send select to audio thread");
        }
    }

    pub fn is_stopped(&self) -> bool {
        match self.track_state {
            TrackState::Stopped => true,
            _ => false,
        }
    }

    pub fn seek_to(&mut self, seek_to_timestamp: u64) {
        self.seek_to_timestamp = seek_to_timestamp;
        self.audio_tx
            .send(AudioCommand::Seek(seek_to_timestamp))
            .expect("Failed to send seek to audio thread");
    }

    // TODO: Should return Result
    pub fn stop(&mut self) {
        match &self.track_state {
            TrackState::Playing | TrackState::Paused => {
                self.track_state = TrackState::Stopped;
                self.audio_tx
                    .send(AudioCommand::Stop)
                    .expect("Failed to send stop to audio thread");
            }
            _ => (),
        }
    }

    // TODO: Should return Result
    pub fn play(&mut self) {
        if let Some(_selected_track) = &self.selected_track {
            match self.track_state {
                TrackState::Unstarted | TrackState::Stopped | TrackState::Playing => {
                    self.track_state = TrackState::Playing;

                    self.audio_tx
                        .send(AudioCommand::Play)
                        .expect("Failed to send play to audio thread");
                }
                TrackState::Paused => {
                    self.track_state = TrackState::Playing;
                    self.audio_tx
                        .send(AudioCommand::Play)
                        .expect("Failed to send play to audio thread");
                }
            }
        }
    }

    // TODO: Should return result
    pub fn pause(&mut self) {
        match self.track_state {
            TrackState::Playing => {
                self.track_state = TrackState::Paused;
                self.audio_tx
                    .send(AudioCommand::Pause)
                    .expect("Failed to send pause to audio thread");
            }
            TrackState::Paused => {
                self.track_state = TrackState::Playing;
                self.audio_tx
                    .send(AudioCommand::Play)
                    .expect("Failed to send play to audio thread");
            }
            _ => (),
        }
    }

    pub fn previous(&mut self, playlist: &Playlist) {
        if let Some(selected_track) = &self.selected_track {
            if let Some(current_track_position) = playlist.get_pos(&selected_track) {
                if current_track_position > 0 {
                    let previous_track = &playlist.tracks[current_track_position - 1];
                    self.select_track(Some((*previous_track).clone()));
                    self.play();
                }
            }
        }
    }

    pub fn next(&mut self, playlist: &Playlist) {
        if let Some(selected_track) = &self.selected_track {
            if let Some(current_track_position) = playlist.get_pos(&selected_track) {
                if current_track_position < playlist.tracks.len() - 1 {
                    let next_track = &playlist.tracks[current_track_position + 1];
                    self.select_track(Some((*next_track).clone()));
                    self.play();
                }
            }
        }
    }

    // TODO - Need to only send message when volume has changed
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        self.audio_tx
            .send(AudioCommand::SetVolume(volume))
            .expect("Failed to send play to audio thread");
    }

    pub fn set_seek_to_timestamp(&mut self, seek_to_timestamp: u64) {
        self.seek_to_timestamp = seek_to_timestamp;
    }

    pub fn set_duration(&mut self, duration: u64) {
        self.duration = duration;
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
