use crate::stuff::library::LibraryItem;
use crate::stuff::playlist::Playlist;
use rodio::cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rodio::cpal::{Data, Sample, SampleFormat};
use rodio::{Decoder, OutputStream, source::Source, Sink};

pub struct Player {
    pub track_state: TrackState,
    pub selected_track: Option<LibraryItem>,
    pub sink: Option<rodio::Sink>,
    pub stream_handle: Option<rodio::OutputStreamHandle>,
    //pub sink: rodio::Sink,
    //pub stream_handle: rodio::OutputStreamHandle,
    pub volume: f32,
}

impl Default for Player { 
    fn default() -> Self {
        tracing::info!("Called player default");
        /*
        let audio_host = rodio::cpal::default_host();
        let audio_device = audio_host.default_output_device().expect("failed to get default output device");
        let (_stream, stream_handle) = rodio::OutputStream::try_from_device(&audio_device).unwrap();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();
        */
        Self {
            track_state: TrackState::Unstarted,
            selected_track: None,
            sink: None,
            stream_handle: None,
            //sink: sink,
            //stream_handle: stream_handle,
            volume: 1.0,
        }
    }
}

impl Player {
    /*
    pub fn new() -> Self {
        Self {
            track_state: TrackState::Unstarted,
            selected_track: None,
            sink: None,
            stream_handle: None,
            volume: 1.0,
        }
    }
    */

    pub fn new(sink: rodio::Sink, stream_handle: rodio::OutputStreamHandle) -> Self {
        Self {
            track_state: TrackState::Unstarted,
            selected_track: None,
            sink: Some(sink),
            stream_handle: Some(stream_handle),
            //sink: sink,
            //stream_handle: stream_handle,
            volume: 1.0,
        }
    }

    pub fn sink_len(&self) -> usize {
        self.sink.as_ref().unwrap().len()
        //self.sink.len()
    }

    pub fn set_sink_and_stream(&mut self, sink: rodio::Sink, stream_handle: rodio::OutputStreamHandle) {
        self.sink = Some(sink);
        self.stream_handle = Some(stream_handle);
        //self.sink = sink;
        //self.stream_handle = stream_handle;
    }

    pub fn set_stream_handle(&mut self, stream_handle: rodio::OutputStreamHandle) {
        self.stream_handle = Some(stream_handle);
        //self.stream_handle = stream_handle;
    }

    pub fn set_sink(&mut self, sink: rodio::Sink) {
        self.sink = Some(sink);
        //self.sink = sink;
    }

    pub fn is_stopped(&self) -> bool {
        match self.track_state {
            TrackState::Stopped => true,
            _ => false,
        }
    }

    pub fn stop(&mut self, sink: &rodio::Sink) {
        match &self.track_state {
            TrackState::Playing | TrackState::Paused => {
                self.track_state = TrackState::Stopped;
                //self.sink.as_ref().unwrap().stop();
                //self.sink.stop();
                sink.stop();
            }
            _ => (),
        }
    }

    pub fn play(&mut self, sink: &rodio::Sink) {
        if let Some(selected_track) = &self.selected_track {

            match self.track_state {
                TrackState::Unstarted | TrackState::Stopped | TrackState::Playing => {
                    self.track_state = TrackState::Playing;

                    let file = std::io::BufReader::new(
                        std::fs::File::open(&selected_track.path()).expect("Failed to open file"),
                    );
                    let source = rodio::Decoder::new(file).expect("Failed to decode audio file");

                    /*
                    let audio_host = rodio::cpal::default_host();
                    let audio_device = audio_host.default_output_device().expect("failed to get default output device");
                    tracing::info!("Play - Audio Device Name: {}", &audio_device.name().unwrap_or("failed to obtain device name".to_string()));
                    let (_stream, rodio_stream_handle) = rodio::OutputStream::try_from_device(&audio_device).unwrap();

                    let sink_try = rodio::Sink::try_new(&rodio_stream_handle);
                    */

                    //let sink_try = rodio::Sink::try_new(self.stream_handle.as_ref().unwrap());

                    //let sink_try = rodio::Sink::try_new(&self.stream_handle);S

                    //self.sink.append(source);
                    //self.sink.as_ref().unwrap().append(source);
                    //
                    //sink.append(source);

                    if let Some(sh) = self.stream_handle.as_ref() {
                        sh.play_raw(source.convert_samples());
                    } else {
                        tracing::warn!("no stream handle found!");
                    }

                    tracing::info!("sleeping for 5 seconds");
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    tracing::info!("done sleeping for 5 seconds");


                    /*
                    match sink_try {
                        Ok(sink) => {
                            tracing::info!("found a sink, now appending the source");
                            /*
                            self.sink = Some(sink);
                            if let Some(sink) = self.sink.as_ref() {
                                sink.append(source);
                            }
                            */
                            self.sink = sink;
                            self.sink.append(source);

                            tracing::info!("sleeping for 5 seconds");
                            std::thread::sleep(std::time::Duration::from_secs(5));
                            tracing::info!("done sleeping for 5 seconds");
                        }
                        Err(e) => tracing::error!("{:?}", e),
                    }
                    */
                }
                TrackState::Paused => {
                    self.track_state = TrackState::Playing;
                    //self.sink.as_ref().unwrap().play();
                    //self.sink.play();
                    sink.play();
                }
            }
        }
    }

    // Toggle pause between paused and playing.
    pub fn pause(&mut self, sink: &rodio::Sink) {
        match self.track_state {
            TrackState::Playing => {
                self.track_state = TrackState::Paused;
                //self.sink.as_ref().unwrap().pause();
                //self.sink.pause();
                sink.pause();
            }
            TrackState::Paused => {
                self.track_state = TrackState::Playing;
                //self.sink.as_ref().unwrap().play();
                //self.sink.play();
                sink.play();
            }
            _ => (),
        }
    }

    pub fn previous(&mut self, playlist: &Playlist, sink: &rodio::Sink) {
        if let Some(selected_track) = &self.selected_track {
            if let Some(current_track_position) = playlist.get_pos(&selected_track) {
                if current_track_position > 0 {
                    let previous_track = &playlist.tracks[current_track_position - 1];
                    self.selected_track = Some(previous_track.clone());
                    self.play(sink);
                }
            }
        }
    }

    pub fn next(&mut self, playlist: &Playlist, sink: &rodio::Sink) {
        if let Some(selected_track) = &self.selected_track {
            if let Some(current_track_position) = playlist.get_pos(&selected_track) {
                if current_track_position < playlist.tracks.len() - 1 {
                    let next_track = &playlist.tracks[current_track_position + 1];
                    self.selected_track = Some(next_track.clone());
                    self.play(sink);
                }
            }
        }
    }

    pub fn set_volume(&mut self, volume: f32, sink: &rodio::Sink) {
        self.volume = volume;
        //self.sink.as_ref().unwrap().set_volume(volume);
        //self.sink.set_volume(volume);
        sink.set_volume(volume);
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
