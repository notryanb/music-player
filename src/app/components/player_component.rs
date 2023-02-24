use super::AppComponent;
use crate::app::App;
use std::sync::atomic::Ordering::Relaxed;

pub struct PlayerComponent;

impl AppComponent for PlayerComponent {
    type Context = App;

    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui) {
        ui.horizontal(|ui| {
            let sample_rate = 44_100; // This is bad. I should be storing this per track

            let cursor = &ctx.player.as_ref().unwrap().cursor.load(Relaxed);
            let current_seconds = (*cursor as f32 / sample_rate as f32) as u32;
            ctx.player
                .as_mut()
                .unwrap()
                .set_seek_in_seconds(current_seconds);

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

            // Time Slider
            let mut seek_in_seconds = ctx.player.as_ref().unwrap().seek_in_seconds;
            let time_slider = ui.add(
                eframe::egui::Slider::new(&mut seek_in_seconds, 0..=(10 * 60))
                    .logarithmic(false)
                    .show_value(true)
                    .clamp_to_range(true),
            );
            ctx.player
                .as_mut()
                .unwrap()
                .set_seek_in_seconds(seek_in_seconds);

            if time_slider.drag_released() {
                ctx.player.as_mut().unwrap().seek_to(seek_in_seconds);
            }

            if let Some(_selected_track) = &ctx.player.as_mut().unwrap().selected_track {
                if stop_btn.clicked() {
                    ctx.player.as_mut().unwrap().stop();
                }

                if play_btn.clicked() {
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
        });
    }
}
