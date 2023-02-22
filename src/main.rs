pub use crate::app::player::Player;
pub use crate::app::App;
pub use crate::app::*;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use eframe::egui;
use minimp3::{Decoder, Frame};
use std::fs::File;
use std::io::{Read, Seek};
use std::sync::atomic::{AtomicU32, Ordering::*};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use rubato::{Resampler, SincFixedIn, InterpolationType, InterpolationParameters, WindowFunction};

mod app;

// TODO:
// Spawn a dedicated audio thread which will hold onto a receiver.
// the audio thread should have an audio buffer
// The receiver should be listening for commands
// Commands can be load audio, scrub, flush buffer, etc..
// the audio thread will need to process the command and fill the audio buffer
//

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

pub struct AudioPlayer {
    state: PlayerState,
    //buffer: Vec<i16>, // This will eventually need to be generic
    //buffer_len: usize,
    buffer_cursor: usize,
}

impl AudioPlayer {
    fn new() -> Self {
        let default_buffer_size = 1024;
        Self {
            state: PlayerState::Stopped,
            //buffer: vec![0; default_buffer_size];
            //buffer_len: default_buffer_size,
            buffer_cursor: 0,
        }
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("App booting...");

    let (tx, rx) = channel();
    let (audio_tx, audio_rx) = channel();
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    let player = Player::new(sink, stream_handle);

    let mut app = App::load().unwrap_or_default();
    app.player = Some(player);
    app.library_sender = Some(tx);
    app.library_receiver = Some(rx);
    app.audio_sender = Some(audio_tx);

    let audio_thread = thread::spawn(move || {
        // This is basically going to be an audio engine completely decoupled from the GUI app and
        // expects commands as input
        // state
        // option current track
        // audio result sender which can send back current playing data
        // Needs the mp3 Decoder
        // CPAL audio system
        // Command engine
        // playback cursor
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let mut supported_config_types = device
            .supported_output_configs()
            .expect("Error querying output config");
        let supported_config = supported_config_types
            .next()
            .expect("Hmm.... no output support config?")
            .with_max_sample_rate();

        let output_err_fn = |err| eprintln!("an error occurred in the output audio stream {}", err);
        let sample_format = supported_config.sample_format();
        let config: cpal::StreamConfig = supported_config.into();
        let config_sample_rate = config.sample_rate.0 as f32;
        let desired_sample_rate = 44_100;
        println!("config sample rate: {config_sample_rate}");

            

        // it looks like anything i want to modify between the cpal thread and here needs
        // to go into a threadsafe struct

        let mut audio_cursor: usize = 0;
        let cursor = Arc::new(AtomicU32::new(0));

        //let mut all_mp3_samples: Vec<i16> = Vec::new();

        let mut audio_output_stream = None;
        let mut state = Arc::new(Mutex::new(PlayerState::Stopped));

        // Audio processing loop
        loop {
            let result = audio_rx.try_recv();
            match result {
                Ok(cmd) => {
                    match cmd {
                        AudioCommand::Stop => {
                            println!("STOP command");
                            {
                                let mut state_guard = state.lock().unwrap();
                                *state_guard = PlayerState::Stopped;
                            }
                            cursor.swap(0, Relaxed);
                        }
                        AudioCommand::Pause => {
                            println!("STOP command");
                            let mut state_guard = state.lock().unwrap();
                            *state_guard = PlayerState::Paused;
                        }
                        AudioCommand::LoadFile(path) => {
                            let buf = std::io::BufReader::new(
                                File::open(&path).expect("couldn't open file"),
                            );
                            let mp3_decoder = Mp3Decoder::new(buf)
                                .expect("Failed to create mp3 decoder from file buffer");
                            let mut all_mp3_samples_f64 = mp3_decoder.map(|s| s as f64).collect::<Vec<f64>>();
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
                            println!("About to resample");
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
                                2
                            ).unwrap();

                            let channels = vec![left, right];
                            let resampled_audio = resampler.process(&channels, None).expect("failed to resample audio");
                            println!("Finished resampling");

                            let zip_audio = resampled_audio[0]
                                .iter()
                                .zip(&resampled_audio[1])
                                .collect::<Vec<(&f64, &f64)>>();

                            let mut vec_channels = vec![];
                            for z in zip_audio {
                                vec_channels.push(vec![*z.0, *z.1]);
                            }

                            let mut flat_channels = vec_channels
                                .iter()
                                .flatten()
                                .map(|s| *s as i16)
                                .collect::<Vec<i16>>();
                                //.into_iter();

                            let sample_count_resampled = *(&flat_channels.len()) as f32 / 2.0f32;
                            let track_length_in_seconds_resampled = sample_count_resampled / desired_sample_rate as f32;
                            println!("resampled: sample_rate: {desired_sample_rate}, sample_count: {sample_count_resampled}, track_length_in_seconds: {track_length_in_seconds_resampled}");

                            // Setup playing
                            let c1 = cursor.clone();
                            let s1 = state.clone();
                            let mut next_sample = move || {
                                // check the state of player...
                                // if playing, fetch_add the cursor
                                // if stopped , return 0s and reset the cursor
                                // if paused, return 0's, don't mutate the cursor
                                let mut state_guard = s1.lock().unwrap();
                                match *state_guard {
                                    PlayerState::Paused => 0i16,
                                    PlayerState::Stopped => 0i16,
                                    PlayerState::Playing => {
                                        let c = c1.fetch_add(1, Relaxed);
                                        //let sample = &all_mp3_samples[c as usize];
                                        let sample = flat_channels[c as usize];
                                        //audio_cursor += 1;
                                        sample
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

                            /*
                            println!(
                                "sample_rate: {}, sample_count: {}, track_length_in_seconds: {}",
                                &config_sample_rate, &sample_count, &track_length_in_seconds
                            );
                            */

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

                            &output_stream.play().unwrap();
                            audio_output_stream = Some(output_stream); // Assign it to variable
                                                                       // that doesn't go out of
                                                                       // scope at the end of the
                                                                       // loop
                            {
                                let mut state_guard = state.lock().unwrap();
                                *state_guard = PlayerState::Playing;
                            }
                            println!("playing");
                        }
                        _ => println!("Something else"),
                    }
                }
                Err(_e) => (), //println!("{e:?}!"),
            }
        }
    });
    //app.audio_thread = Some(t);

    let mut window_options = eframe::NativeOptions::default();
    window_options.initial_window_size = Some(egui::Vec2::new(1024., 768.));
    eframe::run_native("Music Player", window_options, Box::new(|_| Box::new(app)))
        .expect("eframe failed: I should change main to return a result and use anyhow");
}
