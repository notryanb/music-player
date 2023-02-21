use super::AppComponent;
use crate::app::App;

pub struct PlayerComponent;

impl AppComponent for PlayerComponent {
    type Context = App;

    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui) {
        use crate::AudioCommand;
        ui.horizontal(|ui| {
            let stop_btn = ui.button("■");
            let play_btn = ui.button("▶");
            let pause_btn = ui.button("⏸");
            let prev_btn = ui.button("|◀");
            let next_btn = ui.button("▶|");

            let mut volume = ctx.player.as_ref().unwrap().volume;
            ui.add(
                eframe::egui::Slider::new(&mut volume, (0.0 as f32)..=(5.0 as f32))
                    .logarithmic(false)
                    .show_value(false)
                    .clamp_to_range(true),
            );
            ctx.player.as_mut().unwrap().set_volume(volume);

            // DEMO: Making sure clicking the play button actually sends a message to the audio
            // thread.
            if let Some(selected_track) = &ctx.player.as_ref().unwrap().selected_track {
                if stop_btn.clicked() {
                    println!("about to send stop message...");
                    let tx = ctx.audio_sender.as_ref().unwrap().clone();
                    tx.send(AudioCommand::Stop)
                        .expect("Failed to send stop to audio thread");
                }

                if play_btn.clicked() {
                    println!("about to send play message...");
                    let tx = ctx.audio_sender.as_ref().unwrap().clone();
                    let track_path = selected_track.path().clone();
                    tx.send(AudioCommand::LoadFile(track_path))
                        .expect("Failed to send to audio thread");
                }

                if pause_btn.clicked() {
                    println!("about to send pause message...");
                    let tx = ctx.audio_sender.as_ref().unwrap().clone();
                    tx.send(AudioCommand::Pause)
                        .expect("Failed to send pause to audio thread");
                }
            }

            /*
            if let Some(_selected_track) = &ctx.player.as_mut().unwrap().selected_track {
                if stop_btn.clicked() {
                    ctx.player.as_mut().unwrap().stop();
                }

                if play_btn.clicked() {
                    let tx = ctx.audio_sender.as_ref().unwrap().clone();
                    let track_path = _selected_track.path().clone();
                    tx.send(AudioCommand::LoadFile(track_path)).expect("Failed to send to audio thread");
                    ctx.player.as_mut().unwrap().play();
                }

                if pause_btn.clicked() {
                    ctx.player.as_mut().unwrap().pause();
                }

                if prev_btn.clicked() {
                    ctx.player
                        .as_mut()
                        .unwrap()
                        .previous(&ctx.playlists[(ctx.current_playlist_idx).unwrap()])
                }

                if next_btn.clicked() {
                    ctx.player
                        .as_mut()
                        .unwrap()
                        .next(&ctx.playlists[(ctx.current_playlist_idx).unwrap()])
                }
            }
            */
        });
    }
}
