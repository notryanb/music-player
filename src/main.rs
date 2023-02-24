pub use crate::app::player::Player;
pub use crate::app::App;
pub use crate::app::*;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use eframe::egui;
use minimp3::{Decoder, Frame};
use rubato::{InterpolationParameters, InterpolationType, Resampler, SincFixedIn, WindowFunction};
use std::fs::File;
use std::io::{Read, Seek};
use std::sync::atomic::{AtomicU32, Ordering::*};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

mod app;

pub struct Mp3Decoder<R>
where
    R: Read + Seek,
{
    decoder: Decoder<R>,
    current_frame: Frame,
    current_frame_offset: usize,
}

impl<R> Mp3Decoder<R>
where
    R: Read + Seek,
{
    fn new(data: R) -> Result<Self, ()> {
        let mut decoder = Decoder::new(data);
        let current_frame = decoder
            .next_frame()
            .map_err(|_| ())
            .expect("Couldn't get next frame?");
        Ok(Self {
            decoder: decoder,
            current_frame: current_frame,
            current_frame_offset: 0,
        })
    }
}

// TODO
// Instead of decoding the entire buffer up front and then resampling the entire thing,
// I can decode the next frame of data, resample the frame only. One issue here is I'll be missing
// out on being able to figure out track length by frames.
impl<R> Iterator for Mp3Decoder<R>
where
    R: Read + Seek,
{
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        if self.current_frame_offset == self.current_frame.data.len() {
            match self.decoder.next_frame() {
                Ok(frame) => self.current_frame = frame,
                _ => return None,
            }
            self.current_frame_offset = 0;
        }

        let v = self.current_frame.data[self.current_frame_offset];
        self.current_frame_offset += 1;

        Some(v)
    }
}

fn write_sample<T: cpal::Sample>(data: &mut [T], next_sample: &mut dyn FnMut() -> i16) {
    for frame in data.chunks_mut(1) {
        let value = cpal::Sample::from::<i16>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

pub enum PlayerState {
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

    let mut app = App::load().unwrap_or_default();
    app.player = Some(player);
    app.library_sender = Some(tx);
    app.library_receiver = Some(rx);

    let _audio_thread = thread::spawn(move || {
        let cursor = cursor_clone;
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let mut supported_config_types = device
            .supported_output_configs()
            .expect("Error querying output config");
        let supported_config = supported_config_types
            .next()
            .expect("Hmm.... no output support config?")
            .with_max_sample_rate();

        let output_err_fn =
            |err| tracing::error!("an error occurred in the output audio stream {}", err);
        let sample_format = supported_config.sample_format();
        let config: cpal::StreamConfig = supported_config.into();
        let config_sample_rate = config.sample_rate.0 as f32;
        let desired_sample_rate = 44_100;
        tracing::info!("config sample rate: {config_sample_rate}");

        let current_track_sample_rate = desired_sample_rate;

        let mut audio_output_stream = None;
        let state = Arc::new(Mutex::new(PlayerState::Stopped));

        // Audio processing loop
        loop {
            let result = audio_rx.try_recv();
            match result {
                Ok(cmd) => {
                    match cmd {
                        AudioCommand::Seek(seconds) => {
                            let sample_num = current_track_sample_rate * seconds as u32;
                            cursor.swap(sample_num, Relaxed);
                        }
                        AudioCommand::Stop => {
                            tracing::info!("Processing STOP command");
                            {
                                let mut state_guard = state.lock().unwrap();
                                *state_guard = PlayerState::Stopped;
                            }
                            cursor.swap(0, Relaxed);
                        }
                        AudioCommand::Pause => {
                            tracing::info!("Processing PAUSE command");
                            let mut state_guard = state.lock().unwrap();
                            *state_guard = PlayerState::Paused;
                        }
                        AudioCommand::Play => {
                            tracing::info!("Processing PLAY command");
                            let mut state_guard = state.lock().unwrap();
                            *state_guard = PlayerState::Playing;
                        }
                        AudioCommand::LoadFile(path) => {
                            let buf = std::io::BufReader::new(
                                File::open(&path).expect("couldn't open file"),
                            );
                            let mp3_decoder = Mp3Decoder::new(buf)
                                .expect("Failed to create mp3 decoder from file buffer");
                            let all_mp3_samples_f64 =
                                mp3_decoder.map(|s| s as f64).collect::<Vec<f64>>();
                            /*
                            let all_mp3_samples = mp3_decoder.map(|s| s as i16).collect::<Vec<i16>>();
                            let sample_count = all_mp3_samples.len() as f32 / 2.0f32;
                            let track_length_in_seconds = sample_count / desired_sample_rate as f32;
                            */
                            //let mut samples_iter = all_mp3_samples.into_iter();

                            // Resample Audio
                            let mut left = vec![];
                            let mut right = vec![];
                            for (idx, sample) in all_mp3_samples_f64.iter().enumerate() {
                                if idx % 2 == 0 {
                                    left.push(*sample);
                                } else {
                                    right.push(*sample);
                                }
                            }

                            assert!(left.len() == right.len());
                            tracing::info!("About to resample");
                            let params = InterpolationParameters {
                                sinc_len: 256,
                                f_cutoff: 0.95,
                                interpolation: InterpolationType::Linear,
                                oversampling_factor: 256,
                                window: WindowFunction::BlackmanHarris2,
                            };

                            let mut resampler = SincFixedIn::<f64>::new(
                                48_000 as f64 / desired_sample_rate as f64, // should be using config_sample_rate / the mp3's sample_rate
                                2.0,
                                params,
                                left.len(),
                                2,
                            )
                            .unwrap();

                            let channels = vec![left, right];
                            let resampled_audio = resampler
                                .process(&channels, None)
                                .expect("failed to resample audio");
                            tracing::info!("Finished resampling");

                            let zip_audio = resampled_audio[0]
                                .iter()
                                .zip(&resampled_audio[1])
                                .collect::<Vec<(&f64, &f64)>>();

                            let mut vec_channels = vec![];
                            for z in zip_audio {
                                vec_channels.push(vec![*z.0, *z.1]);
                            }

                            let flat_channels = vec_channels
                                .iter()
                                .flatten()
                                .map(|s| *s as i16)
                                .collect::<Vec<i16>>();
                            //.into_iter();

                            // I think it makes sense to assert that the buffer len is evenly
                            // divisible by the audio channel count before going forward.
                            // Right now I could care less...
                            let sample_count_resampled = *(&flat_channels.len()) as f32 / 2.0f32;
                            let track_length_in_seconds_resampled =
                                (sample_count_resampled as f32 / desired_sample_rate as f32) as i32;
                            tracing::info!("resampled: sample_rate: {desired_sample_rate}, sample_count: {sample_count_resampled}, track_length_in_seconds: {track_length_in_seconds_resampled}");

                            cursor.swap(0, Relaxed); // Reset the cursor on every file load after the audio buffer is ready

                            // Setup playing
                            // The cursor and state need to be cloned, as they are Arc'd
                            // and need to be accessible via the cpal callback
                            let c1 = cursor.clone();
                            let s1 = state.clone();

                            let mut next_sample = move || {
                                // check the state of player...
                                // if playing, fetch_add the cursor
                                // if stopped , return 0s and reset the cursor
                                // if paused, return 0's, don't mutate the cursor
                                let state_guard = s1.lock().unwrap();
                                match *state_guard {
                                    PlayerState::Paused => 0i16,
                                    PlayerState::Stopped => 0i16,
                                    PlayerState::Playing => {
                                        let c = c1.fetch_add(1, Relaxed);
                                        // Figure out how to wrap all of this as an iter
                                        if (c as usize) < flat_channels.len() {
                                            let sample = flat_channels[c as usize];
                                            sample
                                        } else {
                                            0i16
                                        }

                                        /*
                                        //let a = all_mp3_samples.clone();
                                        match samples_iter.next() {
                                            Some(sample) => sample,
                                            None => 0i16,
                                        }
                                        */
                                    }
                                }
                            };

                            // The actual playing should be done by the player command, but this is
                            // a test.
                            let output_stream = match sample_format {
                                SampleFormat::F32 => device.build_output_stream(
                                    &config,
                                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                                        write_sample(data, &mut next_sample)
                                    },
                                    output_err_fn,
                                ),
                                SampleFormat::I16 => device.build_output_stream(
                                    &config,
                                    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                                        write_sample(data, &mut next_sample)
                                    },
                                    output_err_fn,
                                ),
                                SampleFormat::U16 => device.build_output_stream(
                                    &config,
                                    move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                                        write_sample(data, &mut next_sample)
                                    },
                                    output_err_fn,
                                ),
                            }
                            .expect("Failed to build output stream");

                            // This stream needs to outlive this block, otherwise it'll shut off.
                            // Easiest way to do this is save the stream in a higher scope's state.

                            // This is the first annoying lint I've encountered. The audio output
                            // stream needs to be cached into a variable outside the loop that will
                            // last the duration of the program. This is because the stream will be
                            // dropped and playback will end. This means we're doing a second
                            // assignment before reading the variable, which is fine, but clippy
                            // complains. To get rid of this, make a simple `let` binding right
                            // before.
                            let _ = audio_output_stream;
                            let _ = &output_stream.play().unwrap();
                            audio_output_stream = Some(output_stream);

                            {
                                let mut state_guard = state.lock().unwrap();
                                *state_guard = PlayerState::Playing;
                            }
                        }
                        _ => tracing::info!("Unhandled case in audio command loop"),
                    }
                }
                Err(_) => (), // When no commands are sent, this will evaluate. aka - it is the
                              // common case. No need to print anything
            }
        }
    });

    let mut window_options = eframe::NativeOptions::default();
    window_options.initial_window_size = Some(egui::Vec2::new(1024., 768.));
    eframe::run_native("Music Player", window_options, Box::new(|_| Box::new(app)))
        .expect("eframe failed: I should change main to return a result and use anyhow");
}
