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
            let previous_vol = volume;

            let volume_slider = ui.add(
                eframe::egui::Slider::new(&mut volume, (0.0 as f32)..=(1.0 as f32))
                    .logarithmic(false)
                    .show_value(true)
                    .clamp_to_range(true)
                    .step_by(0.01)
                    .custom_formatter(|num, _| {
                        let db = 20.0 * num.log10();
                        format!("{db:.02}dB")
                    }),
            );

            if volume_slider.dragged() {
                if let Some(is_processing_ui_change) = &ctx.is_processing_ui_change {
                    // Only send if the volume is actually changing
                    if volume != previous_vol {
                        ctx.player
                            .as_mut()
                            .unwrap()
                            .set_volume(volume, is_processing_ui_change);
                    }
                }
            }

            let mut seek_to_timestamp = ctx.player.as_ref().unwrap().seek_to_timestamp;
            let mut duration = ctx.player.as_ref().unwrap().duration;
            let mut sample_rate = ctx.player.as_ref().unwrap().sample_rate;

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
                    UiCommand::SampleRate(sr) => {
                        tracing::info!("Received sample_rate: {}", sr);
                        sample_rate = sr;
                        ctx.player.as_mut().unwrap().set_sample_rate(sr);
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

            let TimeParts {
                hours: _duration_hours,
                minutes: duration_minutes,
                seconds: duration_seconds,
            } = time_parts_from_duration(duration, sample_rate);
            let TimeParts {
                hours: _current_hours,
                minutes: current_minutes,
                seconds: current_seconds,
            } = time_parts_from_duration(seek_to_timestamp, sample_rate);
            ui.label(format!(
                "{:01}:{:02} / {:01}:{:02}",
                current_minutes, current_seconds, duration_minutes, duration_seconds
            ));
        });
    }
}

struct TimeParts {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
}

fn time_parts_from_duration(duration: u64, sample_rate: f32) -> TimeParts {
    let duration_total_secs = duration as f32 / sample_rate;
    let duration_total_min = (duration_total_secs / 60.0).floor();
    let duration_total_hours = (duration_total_min / 60.0).floor();
    let duration_leftover_secs = (duration_total_secs.floor() - duration_total_min) as u32 % 60;

    TimeParts {
        hours: duration_total_hours as u32,
        minutes: duration_total_min as u32,
        seconds: duration_leftover_secs,
    }
}
