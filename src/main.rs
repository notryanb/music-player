pub use crate::app::player::Player;
pub use crate::app::App;
pub use crate::app::*;

use eframe::egui;
use std::sync::mpsc::channel;

mod app;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("App booting...");

    let (tx, rx) = channel();
    // let (audio_tx, audio_rx) = channel();
    // let cursor = Arc::new(AtomicU32::new(0));
    let player = Player::new();

    // App setup
    let mut app = App::load().unwrap_or_default();
    app.player = Some(player);
    app.library_sender = Some(tx);
    app.library_receiver = Some(rx);

    let mut window_options = eframe::NativeOptions::default();
    window_options.initial_window_size = Some(egui::Vec2::new(1024., 768.));
    eframe::run_native("Music Player", window_options, Box::new(|_| Box::new(app)))
        .expect("eframe failed: I should change main to return a result and use anyhow");
}
