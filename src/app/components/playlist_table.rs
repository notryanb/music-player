use super::AppComponent;
use crate::app::App;
use eframe::egui;

pub struct PlaylistTable;

impl AppComponent for PlaylistTable {
    type Context = App;

    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui) {
        if let Some(current_playlist_idx) = &mut ctx.current_playlist_idx {
            egui::Grid::new("playlist")
                .striped(true)
                .min_col_width(25.)
                .show(ui, |ui| {
                    // Header
                    ui.label("Playing");
                    ui.label("#");
                    ui.label("Artist");
                    ui.label("Album");
                    ui.label("Title");
                    ui.label("Genre");
                    ui.end_row();

                    // Rows
                    for track in ctx.playlists[*current_playlist_idx].tracks.iter() {
                        if let Some(selected_track) = &ctx.player.as_ref().unwrap().selected_track {
                            if selected_track == track {
                                ui.label("▶".to_string());
                            } else {
                                ui.label(" ".to_string());
                            }
                        } else {
                            ui.label(" ".to_string());
                        }

                        if let Some(track_number) = &track.track_number() {
                            ui.label(&track_number.to_string());
                        } else {
                            ui.label("");
                        }

                        let artist_label = ui.add(
                            egui::Label::new(&track.artist().unwrap_or("?".to_string()))
                                .sense(egui::Sense::click()),
                        );

                        ui.label(&track.album().unwrap_or("?".to_string()));
                        ui.label(&track.title().unwrap_or("?".to_string()));
                        ui.label(&track.genre().unwrap_or("?".to_string()));

                        // Temporary hack because I don't yet know how to treat an entire Row
                        // as a response
                        if artist_label.double_clicked() {
                            //ctx.player.as_mut().unwrap().selected_track = Some(track.clone());
                            ctx.player
                                .as_mut()
                                .unwrap()
                                .select_track(Some(track.clone()));
                            ctx.player.as_mut().unwrap().play();
                        }

                        if artist_label.clicked() {
                            ctx.player.as_mut().unwrap().selected_track = Some(track.clone());
                        }

                        ui.end_row();
                    }
                });
        }
    }
}
