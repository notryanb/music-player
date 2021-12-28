use crate::stuff::playlist::{Playlist, Track};

pub struct Player {
    pub track_state: TrackState,
    pub selected_track: Option<Track>,
    pub sink: rodio::Sink,
    pub stream_handle: rodio::OutputStreamHandle,
    pub volume: f32,
}

impl Player {
    pub fn new(sink: rodio::Sink, stream_handle: rodio::OutputStreamHandle) -> Self {
        Self {
            track_state: TrackState::Unstarted,
            selected_track: None,
            sink: sink,
            stream_handle: stream_handle,
            volume: 1.0,
        }
    }

    pub fn stop(&mut self) {
        match &self.track_state {
            TrackState::Playing | TrackState::Paused => {
                self.track_state = TrackState::Stopped;
                self.sink.stop();
            }
            _ => (),
        }
    }

    pub fn play(&mut self) {
        if let Some(selected_track) = &self.selected_track {
            let file = std::io::BufReader::new(
                std::fs::File::open(&selected_track.path).expect("Failed to open file"),
            );
            let source = rodio::Decoder::new(file).expect("Failed to decode audio file");

            match self.track_state {
                TrackState::Unstarted | TrackState::Stopped | TrackState::Playing => {
                    self.track_state = TrackState::Playing;

                    let sink_try = rodio::Sink::try_new(&self.stream_handle);

                    match sink_try {
                        Ok(sink) => {
                            self.sink = sink;
                            self.sink.append(source);
                        }
                        Err(e) => tracing::error!("{:?}", e),
                    }
                }
                TrackState::Paused => {
                    self.track_state = TrackState::Playing;
                    self.sink.play();
                }
                _ => (),
            }
        }
    }

    // Toggle pause between paused and playing.
    pub fn pause(&mut self) {
        match self.track_state {
            TrackState::Playing => {
                self.track_state = TrackState::Paused;
                self.sink.pause();
            }
            TrackState::Paused => {
                self.track_state = TrackState::Playing;
                self.sink.play();
            }
            _ => (),
        }
    }
    
    pub fn previous(&mut self, playlist: &Playlist) {
        if let Some(selected_track) = &self.selected_track {
            if let Some(current_track_position) = playlist.get_pos(&selected_track) {
                if current_track_position > 0 {
                    let previous_track = &playlist.tracks[current_track_position - 1];
                    self.selected_track = Some(previous_track.clone());
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
                    self.selected_track = Some(next_track.clone());
                    self.play();
                }
            }
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        self.sink.set_volume(volume);
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
