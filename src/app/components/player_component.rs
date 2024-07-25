use super::AppComponent;
use crate::egui::style::HandleShape;
use crate::{app::App, UiCommand};

pub struct PlayerComponent;

impl AppComponent for PlayerComponent {
    type Context = App;

    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui) {
        ui.horizontal(|ui| {
            let stop_btn = ui.button("■");
            let play_btn = ui.button("▶");
            let pause_btn = ui.button("⏸");
            let prev_btn = ui.button("|◀");
            let next_btn = ui.button("▶|");

            let mut volume = ctx.player.as_ref().unwrap().volume;
            let volume_slider = ui.add(
                eframe::egui::Slider::new(&mut volume, (0.0 as f32)..=(1.0 as f32))
                    .logarithmic(false)
                    .show_value(true)
                    .clamp_to_range(true)
                    .step_by(0.01)
                    .custom_formatter(|num, _| {
                       let db = 20.0 * num.log10();
                       format!("{db:.02}dB")
                    })
            );

            if volume_slider.dragged() {
                ctx.player.as_mut().unwrap().set_volume(volume);
            }

            let mut seek_to_timestamp = ctx.player.as_ref().unwrap().seek_to_timestamp;
            let mut duration = ctx.player.as_ref().unwrap().duration;

            if let Ok(new_seek_cmd) = ctx.player.as_ref().unwrap().ui_rx.try_recv() {
                match new_seek_cmd {
                    UiCommand::CurrentTimestamp(seek_timestamp) => {
                        seek_to_timestamp = seek_timestamp;
                    }
                    UiCommand::TotalTrackDuration(dur) => {
                        tracing::info!("Received Duration: {}", dur);
                        duration = dur;
                        ctx.player.as_mut().unwrap().set_duration(dur);
                    }
                    UiCommand::AudioFinished => {
                        tracing::info!("Track finished, getting next...");

                        ctx.player
                            .as_mut()
                            .unwrap()
                            .next(&ctx.playlists[(ctx.current_playlist_idx).unwrap()]);
                    } //_ => {}
                }
            }

            // Time Slider
            // TODO - use custom_formatter to maybe turn the duration/timestamp into a
            // hr:min:seconds:ms display?
            let time_slider = ui.add(
                eframe::egui::Slider::new(&mut seek_to_timestamp, 0..=duration)
                    .logarithmic(false)
                    .show_value(false)
                    .clamp_to_range(true)
                    .trailing_fill(true)
                    .handle_shape(HandleShape::Rect { aspect_ratio: 0.5 }),
            );

            ctx.player
                .as_mut()
                .unwrap()
                .set_seek_to_timestamp(seek_to_timestamp);

            if time_slider.drag_stopped() {
                ctx.player.as_mut().unwrap().seek_to(seek_to_timestamp);
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
                        .previous(&ctx.playlists[(ctx.current_playlist_idx).unwrap()]);
                }

                if next_btn.clicked() {
                    ctx.player
                        .as_mut()
                        .unwrap()
                        .next(&ctx.playlists[(ctx.current_playlist_idx).unwrap()]);
                }
            }
        });
    }
}
