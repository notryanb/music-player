use eframe::egui;
use std::sync::atomic::Ordering;
use rb::RbConsumer;

use super::{App, UiCommand};
use crate::app::components::{
    footer::Footer, library_component::LibraryComponent, menu_bar::MenuBar,
    player_component::PlayerComponent, playlist_table::PlaylistTable, playlist_tabs::PlaylistTabs,
    scope_component::ScopeComponent, AppComponent,
};
use crate::player::TrackState;

use crate::meter::{Meter, DB_TICKS, DB_SECTIONS, DbMapper};

impl eframe::App for App {
    fn on_exit(&mut self, _ctx: Option<&eframe::glow::Context>) {
        tracing::info!("exiting and saving");
        self.save_state();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        ctx.request_repaint();

        // Main event processing loop
        if let Some(cmd_rx) = &self.ui_rx {
            match cmd_rx.try_recv() {
                Ok(cmd) => match cmd {
                    UiCommand::LibraryAddItem(lib_item) => self.library.add_item(lib_item),
                    UiCommand::LibraryAddView(lib_view) => self.library.add_view(lib_view),
                    UiCommand::LibraryAddPathId(path_id) => {
                        self.library.set_path_to_imported(path_id)
                    },
                    UiCommand::CurrentTimestamp(seek_timestamp) => {
                        self.player.as_mut().unwrap().set_seek_to_timestamp(seek_timestamp);
                    }
                    UiCommand::TotalTrackDuration(dur) => {
                        tracing::info!("Received Duration: {}", dur);
                        self.player.as_mut().unwrap().set_duration(dur);
                    }
                    UiCommand::SampleRate(sr) => {
                        tracing::info!("Received sample_rate: {}", sr);
                        self.player.as_mut().unwrap().set_sample_rate(sr);
                    }
                    UiCommand::AudioFinished => {
                        tracing::info!("Track finished, getting next...");

                        self.player
                            .as_mut()
                            .unwrap()
                            .next(&self.playlists[(self.current_playlist_idx).unwrap()]);
                    },
                },
                Err(_) => (),
            }
        }

        // copy data from the gui ring buffer into a local collection
        // Individual GUI components can now copy the samples at their own cadence
        if let Some(audio_buf) = &self.played_audio_buffer {
            let num_bytes_read = audio_buf.read(&mut self.ui_audio_buffer[..]).unwrap_or(0);
            self.gui_num_bytes_read = num_bytes_read;

            // Set meter values
            let sample_window = (self.rms_meter_window_size_millis as f32 / 1000.0) * self.device_sample_rate;
            
            if (self.meter_samples.len() as f32) < sample_window * 2.0 {
                self.meter_samples.extend(self.ui_audio_buffer.iter().by_ref());
            } else {
                let left_samples_sum = self.meter_samples
                    .iter()
                    .skip(0)
                    .take(sample_window as usize)
                    .step_by(2)
                    .copied()
                    .map(|x| x * x)
                    .sum::<f32>();

                let left_rms = (left_samples_sum / sample_window).sqrt();
                let left_rms_db = 20.0 * left_rms.log10();

                let right_samples_sum = self.meter_samples
                    .iter()
                    .skip(1)
                    .take(sample_window as usize)
                    .step_by(2)
                    .copied()
                    .map(|x| x * x)
                    .sum::<f32>();

                let right_rms = (right_samples_sum / sample_window).sqrt();
                let right_rms_db = 20.0 * right_rms.log10();
                self.rms_meter = [left_rms_db, right_rms_db];
                self.meter_samples.clear();
            }

            if let Some(transport) = &self.player {
                if transport.track_state != TrackState::Playing {
                    self.rms_meter = [f32::NEG_INFINITY, f32::NEG_INFINITY];
                }
            }
        }

        if let Some(selected_track) = &self.player.as_mut().unwrap().selected_track {
            let display = format!(
                "{} - {} [ Music Player ]",
                &selected_track.artist().unwrap_or("?".to_string()),
                &selected_track.title().unwrap_or("?".to_string())
            );

            ctx.send_viewport_cmd(egui::ViewportCommand::Title(display));
        }
        if self.show_preferences_window {
            eframe::egui::Window::new("Preferences")
                .default_width(200.0)
                .default_height(200.0)
                .resizable([false, false])
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.add(egui::Slider::new(&mut self.rms_meter_window_size_millis, 5..=5000).text("RMS Meter Window Size (ms)"));
                });
        }

        egui::TopBottomPanel::top("MusicPlayer").show(ctx, |ui| {
            MenuBar::add(self, ui);
        });

        egui::TopBottomPanel::top("Player").show(ctx, |ui| {
            egui::Frame::new()
                .inner_margin(6)
                .show(ui, |ui| {
                    PlayerComponent::add(self, ui);
                });

            if self.show_oscilloscope {
                if !self.process_gui_samples.load(Ordering::Relaxed) {
                    self.process_gui_samples.store(true, Ordering::Relaxed);
                }

                eframe::egui::Window::new("Oscilloscope")
                    .default_width(600.0)
                    .default_height(400.0)
                    .resizable([true, true])
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ScopeComponent::add(self, ui);
                    });
            }
        });

        egui::TopBottomPanel::bottom("Footer").show(ctx, |ui| {
            Footer::add(self, ui);
        });

        egui::CentralPanel::default().show(ctx, |_ui| {
            egui::SidePanel::left("Library Window")
                .default_width(500.0)
                .show(ctx, |ui| {
                    LibraryComponent::add(self, ui);
                });
        });

        egui::CentralPanel::default().show(ctx, |_ui| {
            egui::TopBottomPanel::top("Playlist Tabs").show(ctx, |ui| {
                PlaylistTabs::add(self, ui);
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(_current_playlist_idx) = &mut self.current_playlist_idx {
                    egui::ScrollArea::both().show(ui, |ui| {
                        PlaylistTable::add(self, ui);
                    });
                }

                // Temporary
                if self.show_oscilloscope {
                    eframe::egui::Window::new("RMS Meter")
                        .default_width(400.0)
                        .default_height(600.0)
                        .resizable([true, true])
                        .collapsible(false)
                        .show(ctx, |ui| {
                            ui.add(
                                Meter::new(&[self.rms_meter[0], self.rms_meter[1]])
                                .with_ticks(&DB_TICKS)
                                .with_sections(&DB_SECTIONS)
                                .with_text_above("RMS")
                                .with_bar_width(10.0)
                                .show_max(true)
                                .with_mapper(&DbMapper),
                            );
                            
                        });
                }
            });
        });
    }
}
