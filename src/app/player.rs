use crate::app::library::LibraryItem;
use crate::app::playlist::Playlist;
use crate::app::AudioOutput;
use crate::app::output;

use crate::AudioCommand;

use std::sync::atomic::AtomicU32;
use std::sync::mpsc::Sender;
use std::sync::Arc;


pub struct Player {
    pub track_state: TrackState,
    pub selected_track: Option<LibraryItem>,
    pub volume: f32,
    pub seek_in_seconds: u32,
    pub audio_output: Option<Box<dyn AudioOutput>>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            track_state: TrackState::Unstarted,
            selected_track: None,
            audio_output: None,
            volume: 1.0,
            seek_in_seconds: 0,
        }
    }

    pub fn select_track(&mut self, track: Option<LibraryItem>) {
        self.selected_track = track;

        if let Some(track) = &self.selected_track {
            // self.audio_tx
            //     .send(AudioCommand::LoadFile(track.path()))
            //     .expect("Failed to send select to audio thread");
        }
    }

    pub fn is_stopped(&self) -> bool {
        match self.track_state {
            TrackState::Stopped => true,
            _ => false,
        }
    }

    pub fn seek_to(&mut self, seconds: u32) {
        self.seek_in_seconds = seconds;
        // self.audio_tx
        //     .send(AudioCommand::Seek(seconds))
        //     .expect("Failed to send seek to audio thread");
    }

    // TODO: Should return Result
    pub fn stop(&mut self) {
        match &self.track_state {
            TrackState::Playing | TrackState::Paused => {
                self.track_state = TrackState::Stopped;
                // self.audio_tx
                //     .send(AudioCommand::Stop)
                //     .expect("Failed to send stop to audio thread");
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

                    let path = _selected_track.path();
                    tracing::info!("About to play: {}", &path.display());


                    let mut hint = Hint::new();
                    let source = Box::new(std::fs::File::open(path).expect("couldn't open file"));
                    let mss = MediaSourceStream::new(source, Default::default());
                    let format_opts = FormatOptions { enable_gapless: true, ..Default::default() };
                    let metadata_opts: MetadataOptions = Default::default();
                    let track = Some(0);
                    let seek = Some(SeekPosition::Time(0.0));

                    match symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts) {
                        Ok(mut probed) => {
                            // Set the decoder options.
                            let decode_opts =
                                DecoderOptions { verify: true, ..Default::default() };
            
                            // Play it!
                            play(probed.format, track, seek, &decode_opts);
                        }
                        Err(err) => {
                            // The input was not supported by any format reader.
                            tracing::info!("the input is not supported: {}", err);
                            // Err(err);
                        }
                    }

                    // self.audio_tx
                    //     .send(AudioCommand::Play)
                    //     .expect("Failed to send play to audio thread");
                }
                TrackState::Paused => {
                    self.track_state = TrackState::Playing;
                    // self.audio_tx
                    //     .send(AudioCommand::Play)
                    //     .expect("Failed to send play to audio thread");
                }
            }
        }
    }

    // TODO: Should return result
    pub fn pause(&mut self) {
        match self.track_state {
            TrackState::Playing => {
                self.track_state = TrackState::Paused;
                // self.audio_tx
                //     .send(AudioCommand::Pause)
                //     .expect("Failed to send pause to audio thread");
            }
            TrackState::Paused => {
                self.track_state = TrackState::Playing;
                // self.audio_tx
                //     .send(AudioCommand::Play)
                //     .expect("Failed to send play to audio thread");
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

    // TODO - vol to dB
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        //self.sink.set_volume(volume);
    }

    pub fn set_seek_in_seconds(&mut self, seek_in_seconds: u32) {
        self.seek_in_seconds = seek_in_seconds;
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



// Symphonia stuff

use symphonia::core::codecs::{DecoderOptions, FinalizeResult, CODEC_TYPE_NULL};
use symphonia::core::errors::{Error, Result};
use symphonia::core::formats::{Cue, FormatOptions, FormatReader, SeekMode, SeekTo, Track};
use symphonia::core::io::{MediaSource, MediaSourceStream, ReadOnlySource};
use symphonia::core::meta::{ColorMode, MetadataOptions, MetadataRevision, Tag, Value, Visual};
use symphonia::core::probe::{Hint, ProbeResult};
use symphonia::core::units::{Time, TimeBase};



enum SeekPosition {
    Time(f64),
    Timetamp(u64),
}

#[derive(Copy, Clone)]
struct PlayTrackOptions {
    track_id: u32,
    seek_ts: u64,
}


pub enum PlayerState {
    Unstarted,
    Stopped,
    Playing,
    Paused,
}

fn play(
    mut reader: Box<dyn FormatReader>,
    track_num: Option<usize>,
    seek: Option<SeekPosition>,
    decode_opts: &DecoderOptions,
) -> Result<i32>  {
    // If the user provided a track number, select that track if it exists, otherwise, select the
    // first track with a known codec.
    let track = track_num
        .and_then(|t| reader.tracks().get(t))
        .or_else(|| first_supported_track(reader.tracks()));

    let mut track_id = match track {
        Some(track) => track.id,
        _ => return Ok(0),
    };

    // If seeking, seek the reader to the time or timestamp specified and get the timestamp of the
    // seeked position. All packets with a timestamp < the seeked position will not be played.
    //
    // Note: This is a half-baked approach to seeking! After seeking the reader, packets should be
    // decoded and *samples* discarded up-to the exact *sample* indicated by required_ts. The
    // current approach will discard excess samples if seeking to a sample within a packet.
    let seek_ts = if let Some(seek) = seek {
        let seek_to = match seek {
            SeekPosition::Time(t) => SeekTo::Time { time: Time::from(t), track_id: Some(track_id) },
            SeekPosition::Timetamp(ts) => SeekTo::TimeStamp { ts, track_id },
        };

        // Attempt the seek. If the seek fails, ignore the error and return a seek timestamp of 0 so
        // that no samples are trimmed.
        match reader.seek(SeekMode::Accurate, seek_to) {
            Ok(seeked_to) => seeked_to.required_ts,
            Err(Error::ResetRequired) => {
                // print_tracks(reader.tracks());
                track_id = first_supported_track(reader.tracks()).unwrap().id;
                0
            }
            Err(err) => {
                // Don't give-up on a seek error.
                tracing::warn!("seek error: {}", err);
                0
            }
        }
    }
    else {
        // If not seeking, the seek timestamp is 0.
        0
    };

     // The audio output device.
     let mut audio_output = None;

     let mut track_info = PlayTrackOptions { track_id, seek_ts };
 
     let result = loop {
         match play_track(&mut reader, &mut audio_output, track_info, decode_opts) {
             Err(Error::ResetRequired) => {
                 // The demuxer indicated that a reset is required. This is sometimes seen with
                 // streaming OGG (e.g., Icecast) wherein the entire contents of the container change
                 // (new tracks, codecs, metadata, etc.). Therefore, we must select a new track and
                 // recreate the decoder.
                //  print_tracks(reader.tracks());
 
                 // Select the first supported track since the user's selected track number might no
                 // longer be valid or make sense.
                 let track_id = first_supported_track(reader.tracks()).unwrap().id;
                 track_info = PlayTrackOptions { track_id, seek_ts: 0 };
             }
             res => break res,
         }
     };
 
     // Flush the audio output to finish playing back any leftover samples.
     if let Some(audio_output) = audio_output.as_mut() {
         audio_output.flush()
     }
 
     result
}

fn play_track(
    reader: &mut Box<dyn FormatReader>,
    audio_output: &mut Option<Box<dyn output::AudioOutput>>,
    play_opts: PlayTrackOptions,
    decode_opts: &DecoderOptions,
) -> Result<i32> {
    // Get the selected track using the track ID.
    let track = match reader.tracks().iter().find(|track| track.id == play_opts.track_id) {
        Some(track) => track,
        _ => return Ok(0),
    };

    // Create a decoder for the track.
    let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, decode_opts)?;

    // Get the selected track's timebase and duration.
    let _tb = track.codec_params.time_base;
    let _dur = track.codec_params.n_frames.map(|frames| track.codec_params.start_ts + frames);

    // Decode and play the packets belonging to the selected track.
    let result = loop {
        // Get the next packet from the format reader.
        let packet = match reader.next_packet() {
            Ok(packet) => packet,
            Err(err) => break Err(err),
        };

        // If the packet does not belong to the selected track, skip it.
        if packet.track_id() != play_opts.track_id {
            continue;
        }

        // //Print out new metadata.
        // while !reader.metadata().is_latest() {
        //     reader.metadata().pop();

        //     if let Some(rev) = reader.metadata().current() {
        //         print_update(rev);
        //     }
        // }

        // Decode the packet into audio samples.
        match decoder.decode(&packet) {
            Ok(decoded) => {
                // If the audio output is not open, try to open it.
                if audio_output.is_none() {
                    // Get the audio buffer specification. This is a description of the decoded
                    // audio buffer's sample format and sample rate.
                    let spec = *decoded.spec();

                    // Get the capacity of the decoded buffer. Note that this is capacity, not
                    // length! The capacity of the decoded buffer is constant for the life of the
                    // decoder, but the length is not.
                    let duration = decoded.capacity() as u64;

                    // Try to open the audio output.
                    audio_output.replace(output::try_open(spec, duration).unwrap());
                }
                else {
                    // TODO: Check the audio spec. and duration hasn't changed.
                }

                // Write the decoded audio samples to the audio output if the presentation timestamp
                // for the packet is >= the seeked position (0 if not seeking).
                if packet.ts() >= play_opts.seek_ts {

                    // TODO - Send the progress back to GUI
                    // if !no_progress {
                    //     print_progress(packet.ts(), dur, tb);
                    // }

                    if let Some(audio_output) = audio_output {
                        audio_output.write(decoded).unwrap()
                    }
                }
            }
            Err(Error::DecodeError(err)) => {
                // Decode errors are not fatal. Print the error message and try to decode the next
                // packet as usual.
                tracing::warn!("decode error: {}", err);
            }
            Err(err) => break Err(err),
        }
    };

    // if !no_progress {
    //     println!();
    // }

    // Return if a fatal error occured.
    ignore_end_of_stream_error(result)?;

    // Finalize the decoder and return the verification result if it's been enabled.
    do_verification(decoder.finalize())
}

fn first_supported_track(tracks: &[Track]) -> Option<&Track> {
    tracks.iter().find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
}

fn ignore_end_of_stream_error(result: Result<()>) -> Result<()> {
    match result {
        Err(Error::IoError(err))
            if err.kind() == std::io::ErrorKind::UnexpectedEof
                && err.to_string() == "end of stream" =>
        {
            // Do not treat "end of stream" as a fatal error. It's the currently only way a
            // format reader can indicate the media is complete.
            Ok(())
        }
        _ => result,
    }
}

fn do_verification(finalization: FinalizeResult) -> Result<i32> {
    match finalization.verify_ok {
        Some(is_ok) => {
            // Got a verification result.
            tracing::info!("verification: {}", if is_ok { "passed" } else { "failed" });

            Ok(i32::from(!is_ok))
        }
        // Verification not enabled by user, or unsupported by the codec.
        _ => Ok(0),
    }
}

