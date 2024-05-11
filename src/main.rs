pub use crate::app::player::Player;
pub use crate::app::App;
pub use crate::app::*;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use cpal::SampleFormat;
use eframe::egui;
use minimp3::{Decoder, Frame};
use ringbuf::HeapRb;
use ringbuf::traits::{Consumer, Producer, Split};
use rubato::Resampler;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering::*};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;

use symphonia::core::codecs::{DecoderOptions, FinalizeResult, CODEC_TYPE_NULL};
use symphonia::core::errors::{Error, Result};
use symphonia::core::formats::{Cue, FormatOptions, FormatReader, SeekMode, SeekTo, Track};
use symphonia::core::io::{MediaSource, MediaSourceStream, ReadOnlySource};
use symphonia::core::meta::{ColorMode, MetadataOptions, MetadataRevision, Tag, Value, Visual};
use symphonia::core::probe::{Hint, ProbeResult};
use symphonia::core::units::{Time, TimeBase};



mod app;
mod output;
mod resampler;

/*
    Notes
    Use symphonia...
    

*/

// The ring buffer should be 1 sec of device audio
// However, this will be different on each system.
// The device sample rate should be passed into to ring buffer
// creation instead of using a const.
const RB_SIZE: usize = 48000;

// pub struct Mp3Decoder<R>
// where
//     R: Read + Seek,
// {
//     decoder: Decoder<R>,
//     current_frame: Frame,
//     current_frame_offset: usize,
//     sample_rate: u32,
// }

// impl<R> Mp3Decoder<R>
// where
//     R: Read + Seek,
// {
//     fn new(data: R) -> Result<Self, ()> {
//         let mut decoder = Decoder::new(data);

//         let current_frame = decoder
//             .next_frame()
//             .map_err(|_| ())
//             .expect("Couldn't get next frame?");

//         let sample_rate = current_frame.sample_rate as u32;

//         Ok(Self {
//             decoder: decoder,
//             current_frame: current_frame,
//             current_frame_offset: 0,
//             sample_rate,
//         })
//     }
// }

// impl<R> Iterator for Mp3Decoder<R>
// where
//     R: Read + Seek,
// {
//     type Item = i16;

//     #[inline]
//     fn next(&mut self) -> Option<i16> {
//         if self.current_frame_offset == self.current_frame.data.len() {
//             match self.decoder.next_frame() {
//                 Ok(frame) => self.current_frame = frame,
//                 _ => return None,
//             }
//             self.current_frame_offset = 0;
//         }

//         let frame_value = self.current_frame.data[self.current_frame_offset];
//         self.current_frame_offset += 1;
//         Some(frame_value)
//     }
// }

// fn read_frames<R: Read + Seek>(input_buffer: &mut R, nbr: usize, channels: usize) -> Vec<Vec<f32>> {
//     let mut buffer = vec![0u8; 4];
//     let mut wfs = Vec::with_capacity(channels);
//     for _chan in 0..channels {
//         wfs.push(Vec::with_capacity(nbr));
//     }
//     let mut value: f32;
//     for _frame in 0..nbr {
//         for wf in wfs.iter_mut().take(channels) {
//             input_buffer.read(&mut buffer).unwrap();
//             value = f32::from_le_bytes(buffer.as_slice().try_into().unwrap()) as f32;
//             wf.push(value);
//         }
//     }

//     wfs
// }

// fn write_frames<W: Write + Seek>(waves: Vec<Vec<f32>>, output_buffer: &mut W, channels: usize) {
//     let nbr = waves[0].len();
//     for frame in 0..nbr {
//         for chan in 0..channels {
//             let value = waves[chan][frame];
//             let bytes = value.to_le_bytes();
//             output_buffer.write(&bytes).unwrap();
//         }
//     }
// }

// fn write_sample<T: SizedSample + FromSample<i16>>(data: &mut [T], next_sample: &mut dyn FnMut() -> i16) {
//     for frame in data.chunks_mut(1) {
//         let value = T::from_sample(next_sample());
//         for sample in frame.iter_mut() {
//             *sample = value;
//         }
//     }
// }

// // TODO - This should probably be an error
// fn load_file(path: PathBuf, device_sample_rate: u32) -> Vec<i16> {
//     tracing::info!("Loading a file - {}", &path.display());

//     let buf = std::io::BufReader::new(File::open(&path).expect("couldn't open file"));
//     let mp3_decoder = Mp3Decoder::new(buf).expect("Failed to create mp3 decoder from file buffer");

//     //let current_track_sample_rate = current_track_sample_rate.clone();
//     let sample_rate = *(&mp3_decoder.sample_rate);

//     /*
//     {
//         let mut guard = current_track_sample_rate.lock().unwrap();
//         *guard = sample_rate;
//     }
//     */

//     let start = std::time::Instant::now();
//     let raw_samples = mp3_decoder
//         .flat_map(|sample| (sample as f32).to_le_bytes())
//         .collect::<Vec<u8>>();

//     let mut input_cursor = std::io::Cursor::new(&raw_samples);
//     let capacity =
//         (*(&raw_samples.len()) as f32 * (device_sample_rate as f32 / sample_rate as f32)) as usize;
//     let mut output_buffer = Vec::with_capacity(capacity);
//     let mut output_cursor = std::io::Cursor::new(&mut output_buffer);
//     let channels = 2;
//     let chunk_size = 1024;
//     let sub_chunks = 2;

//     let mut fft_resampler = rubato::FftFixedIn::<f32>::new(
//         sample_rate as usize,
//         device_sample_rate as usize,
//         chunk_size,
//         sub_chunks,
//         channels,
//     )
//     .unwrap();

//     let num_frames_per_channel = fft_resampler.input_frames_next();
//     let sample_byte_size = 8;
//     let num_chunks = &raw_samples.len() / (sample_byte_size * channels * num_frames_per_channel);

//     for _chunk in 0..num_chunks {
//         let frame_data = read_frames(&mut input_cursor, num_frames_per_channel, channels);
//         let output = fft_resampler.process(&frame_data, None).unwrap();
//         write_frames(output, &mut output_cursor, channels);
//     }

//     let resampled_audio = output_buffer
//         .iter()
//         .as_slice()
//         .chunks(4)
//         .map(|chunk| (f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])) as i16)
//         .collect::<Vec<i16>>();

//     let end = start.elapsed();
//     tracing::info!("Done resampling file: {:?}", end);
//     tracing::info!("Done loading all samples");

//     resampled_audio
// }

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

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("App booting...");

    let (tx, rx) = channel();
    let (audio_tx, audio_rx) = channel();
    let cursor = Arc::new(AtomicU32::new(0));
    let cursor_clone = cursor.clone();
    let player = Player::new(audio_tx, cursor);



    // App setup
    let mut app = App::load().unwrap_or_default();
    app.player = Some(player);
    app.library_sender = Some(tx);
    app.library_receiver = Some(rx);

    // OLD Audio output setup
    let _audio_thread = thread::spawn(move || {
        // let cursor = cursor_clone;
        // let host = cpal::default_host();
        // let device = host.default_output_device().unwrap();
        // tracing::info!("device: {:?}", &device.name());

        // let mut supported_config_types = device
        //     .supported_output_configs()
        //     .expect("Error querying output config");
        // let supported_config = supported_config_types
        //     .next()
        //     .expect("Hmm.... no output support config?")
        //     .with_max_sample_rate();

        // let output_err_fn =
        //     |err| tracing::error!("an error occurred in the output audio stream {}", err);
        // let sample_format = supported_config.sample_format();
        // let config: cpal::StreamConfig = supported_config.into();
        // let device_sample_rate = config.sample_rate.0 as u32;
        // let current_track_sample_rate = Arc::new(Mutex::new(0u32));
        // tracing::info!("config sample rate: {device_sample_rate}");

        let state = Arc::new(Mutex::new(PlayerState::Stopped));

        let state_clone = state.clone();
        // let (mut audio_producer, mut audio_consumer) = HeapRb::<i16>::new(RB_SIZE).split();

        // let mut next_sample = move || {
        //     let state_guard = state_clone.lock().unwrap();
        //     match *state_guard {
        //         PlayerState::Playing => match audio_consumer.try_pop() {
        //             Some(data) => data,
        //             None => 0i16,
        //         },
        //         _ => 0i16,
        //     }
        // };

        // // The actual playing should be done by the player command, but this is
        // // a test.
        // let output_stream = match sample_format {
        //     SampleFormat::F32 => device.build_output_stream(
        //         &config,
        //         move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        //             write_sample(data, &mut next_sample)
        //         },
        //         output_err_fn,
        //         None
        //     ),
        //     SampleFormat::I16 => device.build_output_stream(
        //         &config,
        //         move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
        //             write_sample(data, &mut next_sample)
        //         },
        //         output_err_fn,
        //         None
        //     ),
        //     SampleFormat::U16 => device.build_output_stream(
        //         &config,
        //         move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
        //             write_sample(data, &mut next_sample)
        //         },
        //         output_err_fn,
        //         None
        //     ),
        //     _ => todo!(),
        // }
        // .expect("Failed to build output stream");

        // // This stream needs to outlive this block, otherwise it'll shut off.
        // // Easiest way to do this is save the stream in a higher scope's state.
        // let _ = &output_stream.play().unwrap();

        // let mut audio_source: Option<Vec<i16>> = None;



        'audio_present: loop {               
            match audio_rx.try_recv() {
                Ok(cmd) => {
                    match cmd {
                        AudioCommand::Seek(seconds) => {
                            tracing::info!("Processing SEEK command for {} seconds", seconds);


                            // // TODO - Need to figure out how to implement using iterator + ring buf
                            // let guard = current_track_sample_rate.lock().unwrap();
                            // let sample_num = *guard * seconds as u32;
                            // drop(guard);
                            // cursor.swap(sample_num, Relaxed);
                        }
                        AudioCommand::Stop => {
                            tracing::info!("Processing STOP command");


                            // {
                            //     let mut state_guard = state.lock().unwrap();
                            //     *state_guard = PlayerState::Stopped;
                            // }
                            // // TODO - Doing this incurs an extra allocation.
                            // // Also, when stopping the ring buffer isn't flushed, so when
                            // // you start to play again there is still data to consume from
                            // // the previous track
                            // audio_source = audio_source_clone;
                            break 'audio_present;
                        }
                        AudioCommand::Pause => {
                            tracing::info!("Processing PAUSE command");


                            // let mut state_guard = state.lock().unwrap();
                            // *state_guard = PlayerState::Paused;
                        }
                        AudioCommand::Play => {
                            tracing::info!("Processing PLAY command");


                            // new code
                            let path = "";
                            let mut hint = Hint::new();
                            let source = Box::new(File::open(path).expect("Failed to open fail"));
                            let mss = MediaSourceStream::new(source, Default::default());
                            let format_opts = FormatOptions{ enable_gapless: true, ..Default::default() };
                            let metadata_opts: MetadataOptions = Default::default();
                            
                            // Get the value of the track option, if provided.
                            let track = Some(0);

                            match symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts) {
                                Ok(mut probed) => {
                                    let seek = Some(SeekPosition::Time(0.0));

                                    // Set the decoder options.
                                    let decode_opts = DecoderOptions { verify: true, ..Default::default() };

                                    // Play it!
                                    play(probed.format, track, seek, &decode_opts).expect("failed to play");
                                }
                                Err(err) => {
                                    // The input was not supported by any format reader.
                                    tracing::info!("the input is not supported");
                                    // Err(err);
                                }
                            }


                            // let mut state_guard = state.lock().unwrap();
                            // *state_guard = PlayerState::Playing;
                        }
                        AudioCommand::LoadFile(path) => {
                            // let resampled_audio = load_file(path, device_sample_rate);
                            // audio_source = Some(resampled_audio);
                            tracing::info!("Processing LOAD FILE command for path: {:?}", &path);

                            break 'audio_present;
                        }
                        _ => tracing::warn!("Unhandled case in audio command loop"),
                    }
                }
                Err(_) => (), // When no commands are sent, this will evaluate. aka - it is the
                                // common case. No need to print anything
            }
        }

        // When the audio_source is None.
        // This will only be entered when the app boots and a track has not yet been
        // selected.
        match audio_rx.try_recv() {
            Ok(cmd) => match cmd {
                AudioCommand::LoadFile(path) => {
                    tracing::info!("About to load file: {:?}", &path);





                    // let resampled_audio = load_file(path, device_sample_rate);
                    // audio_source = Some(resampled_audio);
                }
                _ => tracing::warn!("Unhandled case in audio command loop"),
            },
            Err(_) => (), // When no commands are sent, this will evaluate. aka - it is the
                                // common case. No need to print anything
    }}); // Audio Thread end





    let mut window_options = eframe::NativeOptions::default();
    window_options.initial_window_size = Some(egui::Vec2::new(1024., 768.));
    eframe::run_native("Music Player", window_options, Box::new(|_| Box::new(app)))
        .expect("eframe failed: I should change main to return a result and use anyhow");
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
