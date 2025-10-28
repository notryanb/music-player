use super::AppComponent;
use crate::app::App;
use eframe::egui;
use egui_extras::{Column, TableBuilder};

pub struct PlaylistTable;

impl AppComponent for PlaylistTable {
    type Context = App;

    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui) {
        if let Some(current_playlist_idx) = &mut ctx.current_playlist_idx {
            let available_height = ui.available_height();
            let table = TableBuilder::new(ui)
                .striped(false)
                .resizable(true)
                .cell_layout(eframe::egui::Layout::left_to_right(
                    eframe::egui::Align::Center,
                ))
                .column(Column::remainder()) // Playing
                .column(Column::remainder()) // number
                .column(Column::remainder()) // artist
                .column(Column::remainder()) // album
                .column(Column::remainder()) // title
                .column(Column::remainder()) // genre
                .sense(eframe::egui::Sense::click())
                .min_scrolled_height(0.0)
                .max_scroll_height(available_height);

            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Playing");
                    });
                    header.col(|ui| {
                        ui.strong("#");
                    });
                    header.col(|ui| {
                        ui.strong("Artist");
                    });
                    header.col(|ui| {
                        ui.strong("Album");
                    });
                    header.col(|ui| {
                        ui.strong("Title");
                    });
                    header.col(|ui| {
                        ui.strong("Genre");
                    });
                })
                .body(|mut body| {
                    let mut track_to_remove = None;

                    for (track_idx, track) in ctx.playlists[*current_playlist_idx]
                        .tracks
                        .iter()
                        .enumerate()
                    {
                        body.row(20.0, |mut row| {
                            // Playing
                            if let Some(selected_track) =
                                &ctx.player.as_ref().unwrap().selected_track
                            {
                                if selected_track == track {
                                    row.set_selected(true);
                                    row.col(|ui| {
                                        ui.label("▶".to_string());
                                    });
                                } else {
                                    row.col(|ui| {
                                        ui.label(" ".to_string());
                                    });
                                }
                            } else {
                                row.col(|ui| {
                                    ui.label(" ".to_string());
                                });
                            }

                            // Track No.
                            if let Some(track_number) = &track.track_number() {
                                row.col(|ui| {
                                    ui.label(&track_number.to_string());
                                });
                            } else {
                                row.col(|ui| {
                                    ui.label("".to_string());
                                });
                            }

                            row.col(|ui| {
                                ui.label(&track.artist().unwrap_or("?".to_string()));
                            });
                            row.col(|ui| {
                                ui.label(&track.album().unwrap_or("?".to_string()));
                            });
                            row.col(|ui| {
                                ui.label(&track.title().unwrap_or("?".to_string()));
                            });
                            row.col(|ui| {
                                ui.label(&track.genre().unwrap_or("?".to_string()));
                            });

                            if row.response().double_clicked() {
                                ctx.player
                                    .as_mut()
                                    .unwrap()
                                    .select_track(Some(track.clone()));
                                ctx.player.as_mut().unwrap().play();
                            }

                            // TODO - If I decide to keep this, there needs to be some
                            // difference between the selected track and currently playing track
                            // if row.response().clicked() {
                            //     ctx.player.as_mut().unwrap().selected_track = Some(track.clone());
                            // }

                            if row.response().clicked_by(egui::PointerButton::Secondary) {
                                // TODO: send a msg to remove the track from the playlist
                                // This should ideally be handled by a UI Command processor
                                track_to_remove = Some(track_idx);
                            }
                        })
                    }

                    // We can't remove the track from the playlist while it is iterating
                    if let Some(remove_id) = track_to_remove {
                        ctx.playlists[*current_playlist_idx].remove(remove_id);
                    }
                });
        }
    }
}
