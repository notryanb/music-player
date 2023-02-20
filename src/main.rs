pub use crate::app::player::Player;
pub use crate::app::App;

use eframe::egui;
use std::sync::mpsc::channel;
use std::thread;

mod app;

// TODO:
// Spawn a dedicated audio thread which will hold onto a receiver.
// the audio thread should have an audio buffer 
// The receiver should be listening for commands
// Commands can be load audio, scrub, flush buffer, etc..
// the audio thread will need to process the command and fill the audio buffer

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

    // have the audio thread spawn here
    // and have it capture the receiver
    let t = thread::spawn(move || {
        // This is basically going to be an audio engine completely decoupled from the GUI app and 
        // expects commands as input
        // state
        // option current track 
        // audio result sender which can send back current playing data
        // Needs the mp3 Decoder
        // CPAL audio system
        // Command engine
        // playback cursor


        // Audio processing loop
        loop {
            let result = audio_rx.try_recv();
            match result {
                Ok(data) => println!("The data: {data:?}"),
                Err(_e) => (),//println!("{e:?}!"),
            }
        }
    });
    //app.audio_thread = Some(t);

    let mut window_options = eframe::NativeOptions::default();
    window_options.initial_window_size = Some(egui::Vec2::new(1024., 768.));
    eframe::run_native("Music Player", window_options, Box::new(|_| Box::new(app))).expect("eframe failed: I should change main to return a result and use anyhow");
}
