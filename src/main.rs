pub use crate::app::player::Player;
pub use crate::app::App;
pub use crate::app::*;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use eframe::egui;
use minimp3::{Decoder, Frame};
use rubato::{InterpolationParameters, InterpolationType, Resampler, SincFixedIn, WindowFunction};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek};
use std::sync::atomic::{AtomicU32, Ordering::*};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

mod app;

// This holds several buffers
pub struct AudioBank {
    pub buffers: HashMap<usize, AudioBuffer>,
    pub count: usize, // This can be replaced if we randomize the key for the map
    pub currently_playing_buffer: usize, // TODO: Turn the datatype into some sort of key which can be used in the hashmap
}

impl AudioBank {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
            count: 0,
            currently_playing_buffer: 0,
        }
    }

    pub fn insert(&mut self, key: usize, buffer: AudioBuffer) {
        self.buffers.insert(key, buffer);
    }


    // TODO - probably want to lock the buffer for writes by whatever is using this.
    // I think reading while locked for writing is okay...?
    pub fn append_sample(&mut self, key: usize, sample: i16) {
        if let Some(buffer) = self.buffers.get_mut(&key) {
            buffer.append_sample(sample);
        }
    }

    pub fn append_samples(&mut self, key: usize, samples: Vec<i16>) {
        if let Some(buffer) = self.buffers.get_mut(&key) {
            // TODO: probably want to make this one append all instead of individual calls
            for sample in samples {
                buffer.append_sample(sample);
            }
        }
    }

    pub fn get_sample(&self, sample_idx: usize) -> i16 {
        if let Some(buffer) = self.buffers.get(&self.currently_playing_buffer) {
            buffer.get_sample(sample_idx)
        } else {
            0i16
        }
    }

    pub fn select_buffer(&mut self, key: usize) {
        if self.buffers.get(&key).is_some() {
            self.currently_playing_buffer = key;
        }
    }
}

pub struct AudioBuffer {
    pub data: Vec<i16>,
    pub samples_len: usize, // This should be updated by the Mp3Decoder when it is done processing
                            // samples
}

impl AudioBuffer {
    pub fn new() -> Self {
        Self {
            data: vec![0; 1024],
            samples_len: 0,
        }
    }

    // Think about returning an Option....
    pub fn get_sample(&self, idx: usize) -> i16 {
        if idx > &self.data.len() - 1 {
            return 0;
        }
        self.data[idx]
    }

    pub fn append_sample(&mut self, sample: i16) {
        self.data.push(sample);
    }
}

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

        let frame_value = self.current_frame.data[self.current_frame_offset];
        self.current_frame_offset += 1;
        Some(frame_value)
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
        let audio_buffers = Arc::new(Mutex::new(AudioBank::new()));
        let state = Arc::new(Mutex::new(PlayerState::Stopped));

        let c1 = cursor.clone();
        let s1 = state.clone();
        let ab = audio_buffers.clone();
        //let abk = audio_buffers_key.clone();

        let mut next_sample = move || {
            let state_guard = s1.lock().unwrap();
            match *state_guard {
                PlayerState::Playing => {
                    let c = c1.fetch_add(1, Relaxed);
                    {
                        let guard = ab.lock().unwrap();
                        guard.get_sample(c as usize)
                    }
                }
                _ => 0i16,
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
        let _ = &output_stream.play().unwrap();

        {
            let mut state_guard = state.lock().unwrap();
            *state_guard = PlayerState::Playing;
        }

        // Audio processing loop
        loop {
            match audio_rx.try_recv() {
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
                        AudioCommand::Select(key) => {
                            tracing::info!("Processing SELECT {}", &key);
                            let ab = audio_buffers.clone();
                            {
                                let mut guard = ab.lock().unwrap();
                                guard.select_buffer(key);
                            }

                            cursor.swap(0, Relaxed); // Reset the cursor on every file load after the audio buffer is ready
                        }
                        // This shouldn't affect the player state at all.
                        // It only happens when a file is supposed to be loaded...
                        // ideally from adding it to a playlist
                        AudioCommand::LoadFile(path, key) => {
                            tracing::info!("Loading a file - {}", &path.display());

                            let ab = audio_buffers.clone();
                            let buf = std::io::BufReader::new(
                                File::open(&path).expect("couldn't open file"),
                            );
                            let mp3_decoder = Mp3Decoder::new(buf)
                                .expect("Failed to create mp3 decoder from file buffer");

                            let audio_buffer = AudioBuffer::new();

                            // I think it makes sense to generate key + file on the gui side and
                            // pass that in. Otherwise I think everytime this gets called, the same
                            // track will get a new buffer, but we want the existing one.
                            // TODO: - I think it is worth exploring creating audio buffers when
                            // tracks are added to a playlist, not when they're selected. This way
                            // the entire playlist is loaded into memory and the currently playing
                            // buffer is less likely to be starved. When a track is loaded into the
                            // playlist. Think about who creates the key... the AudioBank or the
                            // Player.

                            // TODO: Create debug windows that show buffer stats with progress
                            // bars?
                            {
                                let mut guard = ab.lock().unwrap();
                                let _ = guard.insert(key, audio_buffer);
                            }

                            thread::spawn(move || {
                                let raw_samples = mp3_decoder.collect::<Vec<i16>>();

                                // do resampling
                                let mut left = vec![];
                                let mut right = vec![];
                                for (idx, sample) in raw_samples.iter().enumerate() {
                                    if idx % 2 == 0 {
                                        left.push(*sample as f32);
                                    } else {
                                        right.push(*sample as f32);
                                    }
                                }

                                assert!(left.len() == right.len());

                                let params = InterpolationParameters {
                                    sinc_len: 256,
                                    f_cutoff: 0.95,
                                    interpolation: InterpolationType::Linear,
                                    oversampling_factor: 256,
                                    window: WindowFunction::BlackmanHarris2,
                                };

                                let mut resampler = SincFixedIn::<f32>::new(
                                    48_000 as f64 / 44_100 as f64, // should be using config_sample_rate / the mp3's sample_rate
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

                                let zip_audio = resampled_audio[0]
                                    .iter()
                                    .zip(&resampled_audio[1])
                                    .collect::<Vec<(&f32, &f32)>>();
                                    //.flat_map(|z| vec![*z.0 as i16, *z.1 as i16])
                                    //.collect::<Vec<i16>>();

                                let mut vec_channels = vec![];
                                for z in zip_audio {
                                    vec_channels.push(vec![*z.0, *z.1]);
                                }

                                let flat_channels = vec_channels
                                    .iter()
                                    .flatten()
                                    .map(|s| *s as i16)
                                    .collect::<Vec<i16>>();
                                                                    
                                // append resampled data to audio bank buffer
                                {
                                    let mut guard = ab.lock().unwrap();
                                    guard.append_samples(key, flat_channels);
                                }

                                tracing::info!("Done loading file and resampling");
                            });
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
