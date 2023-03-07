pub use crate::app::player::Player;
pub use crate::app::App;
pub use crate::app::*;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use eframe::egui;
use minimp3::{Decoder, Frame};
use ringbuf::{HeapRb};
use rubato::Resampler;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::sync::atomic::{AtomicU32, Ordering::*};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

mod app;

const RB_SIZE: usize = 48000 * 2;

/*
// TODO - If this works, maybe I should be using the RingBuf directly?
pub struct AudioBuffer {
    audio_producer: Producer<i16, Arc<HeapRb<i16>>>,
    audio_consumer: Consumer<i16, Arc<HeapRb<i16>>>,
}

impl AudioBuffer {
    pub fn new() -> Self {
        let (audio_producer, audio_consumer) = HeapRb::<i16>::new(RB_SIZE).split();
        Self {
           audio_producer,
           audio_consumer,
        }
    }

    // Think about returning an Option....
    pub fn get_sample(&self, idx: usize) -> i16 {
        match self.audio_consumer.pop() {
            Some(data) => data,
            None => 0i16,
        }
        /*
        if idx > &self.data.len() - 1 {
            return 0;
        }
        self.data[idx]
        */
    }

    pub fn append_sample(&mut self, sample: i16) {
        self.audio_producer.push(sample).map_err(|_| ());
        //self.data.push(sample);
    }

    pub fn can_accept_samples(&self) -> bool {
        self.audio_producer.free_len() > 0
    }

    /*
    pub fn set_buffer(&mut self, new_audio_buffer: Vec<i16>) {
        //self.data = new_audio_buffer
    }
    */
}
*/

pub struct Mp3Decoder<R>
where
    R: Read + Seek,
{
    decoder: Decoder<R>,
    current_frame: Frame,
    current_frame_offset: usize,
    sample_rate: u32,
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
        let sample_rate = current_frame.sample_rate as u32;

        Ok(Self {
            decoder: decoder,
            current_frame: current_frame,
            current_frame_offset: 0,
            sample_rate,
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


fn read_frames<R: Read + Seek>(input_buffer: &mut R, nbr: usize, channels: usize) -> Vec<Vec<f32>> {
    let mut buffer = vec![0u8; 4];
    let mut wfs = Vec::with_capacity(channels);
    for _chan in 0..channels {
       wfs.push(Vec::with_capacity(nbr)); 
    }
    let mut value: f32;
    for _frame in 0..nbr {
        for wf in wfs.iter_mut().take(channels) {
            input_buffer.read(&mut buffer).unwrap();
            value = f32::from_le_bytes(buffer.as_slice().try_into().unwrap()) as f32;
            wf.push(value);
        }
    }

    wfs
}

fn write_frames<W: Write + Seek>(waves: Vec<Vec<f32>>, output_buffer: &mut W, channels: usize) {
    let nbr = waves[0].len();
    for frame in 0..nbr {
        for chan in 0..channels {
            let value = waves[chan][frame];
            let bytes = value.to_le_bytes();
            output_buffer.write(&bytes).unwrap();
        }
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

/*
fn write_samples<T: cpal::Sample>(data: &mut [T], next_sample_chunk: &mut dyn FnMut() -> &[i16]) {
    for frame in data.chunks_mut(1024) {
        let values = next_sample_chunk().map(|sample| cpal::Sample::from::<i16>(sample)).collect();
        frame = values;
    }
}
*/

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
        let device_sample_rate = config.sample_rate.0 as u32;
        let current_track_sample_rate = Arc::new(Mutex::new(0u32));
        tracing::info!("config sample rate: {device_sample_rate}");

        let state = Arc::new(Mutex::new(PlayerState::Stopped));

        let state_clone = state.clone();
        let (mut audio_producer, mut audio_consumer) = HeapRb::<i16>::new(RB_SIZE).split();

        let mut next_sample = move || {
            let state_guard = state_clone.lock().unwrap();
            match *state_guard {
                PlayerState::Playing => {
                    match audio_consumer.pop() {
                        Some(data) => data,
                        None => 0i16,
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

        let mut audio_source: Option<Vec<i16>> = None;

        loop {

            if let Some(mut audio_source1) = audio_source {
                let mut audio_source_iter = audio_source1.into_iter();

                loop {
                    let written = audio_producer.push_iter(&mut audio_source_iter);
                    tracing::info!("written: {written}");

                    thread::sleep(std::time::Duration::from_millis(500));

                    match audio_rx.try_recv() {
                        Ok(cmd) => {
                            match cmd {
                                AudioCommand::Seek(seconds) => {
                                    let guard = current_track_sample_rate.lock().unwrap();
                                    let sample_num = *guard * seconds as u32;
                                    drop(guard);
                                    cursor.swap(sample_num, Relaxed);
                                }
                                AudioCommand::Stop => {
                                    tracing::info!("Processing STOP command");
                                    {
                                        let mut state_guard = state.lock().unwrap();
                                        *state_guard = PlayerState::Stopped;
                                    }
                                    tracing::info!("Processed STOP command");
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
                                    tracing::info!("Loading a file - {}", &path.display());

                                    let buf = std::io::BufReader::new(
                                        File::open(&path).expect("couldn't open file"),
                                    );
                                    let mp3_decoder = Mp3Decoder::new(buf)
                                        .expect("Failed to create mp3 decoder from file buffer");

                                    let current_track_sample_rate = current_track_sample_rate.clone();
                                    let sample_rate = *(&mp3_decoder.sample_rate);

                                    {
                                        let mut guard = current_track_sample_rate.lock().unwrap();
                                        *guard = sample_rate; 
                                    }

                                    let start = std::time::Instant::now();
                                    let raw_samples = mp3_decoder
                                        .flat_map(|sample| (sample as f32).to_le_bytes())
                                        .collect::<Vec<u8>>();

                                    let mut input_cursor = std::io::Cursor::new(&raw_samples);
                                    let capacity = (*(&raw_samples.len()) as f32 * (device_sample_rate as f32 / sample_rate as f32)) as usize;
                                    let mut output_buffer = Vec::with_capacity(capacity);
                                    let mut output_cursor = std::io::Cursor::new(&mut output_buffer);
                                    let channels = 2;
                                    let chunk_size = 1024;
                                    let sub_chunks = 2;

                                    let mut fft_resampler = rubato::FftFixedIn::<f32>::new(
                                        sample_rate as usize,
                                        device_sample_rate as usize,
                                        chunk_size,
                                        sub_chunks,
                                        channels
                                    ).unwrap();

                                    let num_frames_per_channel = fft_resampler.input_frames_next();
                                    let sample_byte_size = 8;
                                    let num_chunks = &raw_samples.len() / (sample_byte_size * channels * num_frames_per_channel);

                                    for _chunk in 0..num_chunks {
                                        let frame_data = read_frames(&mut input_cursor, num_frames_per_channel, channels);
                                        let output = fft_resampler.process(&frame_data, None).unwrap();
                                        write_frames(output, &mut output_cursor, channels);
                                    }


                                    let resampled_audio = output_buffer
                                        .iter()
                                        .as_slice()
                                        .chunks(4)
                                        .map(|chunk| {
                                            (f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])) as i16
                                        })
                                        //.into_iter();
                                        .collect::<Vec<i16>>();

                                    let end = start.elapsed();
                                    tracing::info!("Done resampling file: {:?}ms", end);
                                    tracing::info!("Done loading all samples");

                                    audio_source = Some(resampled_audio);
                                    break;
                                }
                                _ => tracing::warn!("Unhandled case in audio command loop"),
                            }
                        }
                        Err(_) => (), // When no commands are sent, this will evaluate. aka - it is the
                                    // common case. No need to print anything
                    }
                }
            } else {
                match audio_rx.try_recv() {
                    Ok(cmd) => {
                        match cmd {
                            AudioCommand::LoadFile(path) => {
                                tracing::info!("Loading a file - {}", &path.display());

                                let buf = std::io::BufReader::new(
                                    File::open(&path).expect("couldn't open file"),
                                );
                                let mp3_decoder = Mp3Decoder::new(buf)
                                    .expect("Failed to create mp3 decoder from file buffer");

                                let current_track_sample_rate = current_track_sample_rate.clone();
                                let sample_rate = *(&mp3_decoder.sample_rate);

                                {
                                    let mut guard = current_track_sample_rate.lock().unwrap();
                                    *guard = sample_rate; 
                                }

                                let start = std::time::Instant::now();
                                let raw_samples = mp3_decoder
                                    .flat_map(|sample| (sample as f32).to_le_bytes())
                                    .collect::<Vec<u8>>();

                                let mut input_cursor = std::io::Cursor::new(&raw_samples);
                                let capacity = (*(&raw_samples.len()) as f32 * (device_sample_rate as f32 / sample_rate as f32)) as usize;
                                let mut output_buffer = Vec::with_capacity(capacity);
                                let mut output_cursor = std::io::Cursor::new(&mut output_buffer);
                                let channels = 2;
                                let chunk_size = 1024;
                                let sub_chunks = 2;

                                let mut fft_resampler = rubato::FftFixedIn::<f32>::new(
                                    sample_rate as usize,
                                    device_sample_rate as usize,
                                    chunk_size,
                                    sub_chunks,
                                    channels
                                ).unwrap();

                                let num_frames_per_channel = fft_resampler.input_frames_next();
                                let sample_byte_size = 8;
                                let num_chunks = &raw_samples.len() / (sample_byte_size * channels * num_frames_per_channel);

                                for _chunk in 0..num_chunks {
                                    let frame_data = read_frames(&mut input_cursor, num_frames_per_channel, channels);
                                    let output = fft_resampler.process(&frame_data, None).unwrap();
                                    write_frames(output, &mut output_cursor, channels);
                                }

                                let resampled_audio = output_buffer
                                    .iter()
                                    .as_slice()
                                    .chunks(4)
                                    .map(|chunk| {
                                        (f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])) as i16
                                    })
                                    //.into_iter();
                                    .collect::<Vec<i16>>();

                                let end = start.elapsed();
                                tracing::info!("Done resampling file: {:?}ms", end);

                                audio_source = Some(resampled_audio);
                                tracing::info!("Done loading all samples");
                            }
                            _ => tracing::warn!("Unhandled case in audio command loop"),
                        }
                    }
                    Err(_) => (), // When no commands are sent, this will evaluate. aka - it is the
                                // common case. No need to print anything
                }
            }
        }
    });

    let mut window_options = eframe::NativeOptions::default();
    window_options.initial_window_size = Some(egui::Vec2::new(1024., 768.));
    eframe::run_native("Music Player", window_options, Box::new(|_| Box::new(app)))
        .expect("eframe failed: I should change main to return a result and use anyhow");
}
