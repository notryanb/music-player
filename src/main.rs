use eframe::{egui};
pub use crate::app::App;
pub use crate::app::player::Player;

mod app;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("App booting...");

    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    let player = Player::new(sink, stream_handle);

    /*

    let mut app = App::load().unwrap_or_default();
    app.set_player(player);
    */


    let app_state = App {
        player: Some(player),
        playlists: Vec::new(),
        current_playlist_idx: None,
        playlist_idx_to_remove: None,
        library: None,
    };


    let mut window_options = eframe::NativeOptions::default();
    window_options.initial_window_size = Some(egui::Vec2::new(1024., 768.));
    eframe::run_native(Box::new(app_state), window_options);
}
