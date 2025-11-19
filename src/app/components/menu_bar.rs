use super::AppComponent;

use crate::app::{library::LibraryPathStatus, App, Playlist};
use egui_extras::{Column, TableBuilder};

pub struct MenuBar;

impl AppComponent for MenuBar {
    type Context = App;

    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui) {
        crate::egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                let _open_btn = ui.button("Open");

                ui.separator();

                let _add_files_btn = ui.button("Add Files");
                let _add_folders_btn = ui.button("Add Folders");

                ui.separator();

                if ui.button("New Playlist").clicked() {
                    let default_name_count = ctx
                        .playlists
                        .iter()
                        .filter(|pl| pl.get_name().unwrap().starts_with("New Playlist"))
                        .count();
                    let playlist_name = match default_name_count {
                        0 => "New Playlist".to_string(),
                        _ => format!("New Playlist ({})", default_name_count - 1),
                    };

                    let mut new_playlist = Playlist::new();
                    new_playlist.set_name(playlist_name);

                    ctx.playlists.push(new_playlist.clone());
                    ctx.current_playlist_idx = Some(ctx.playlists.len() - 1);
                }
                let _load_playlist_btn = ui.button("Load Playlist");
                let _save_playlist_btn = ui.button("Save Playlist");

                ui.separator();

                if ui.button("Preferences").clicked() {
                    ctx.show_preferences_window = !ctx.show_preferences_window;
                };

                ui.separator();

                if ui.button("Exit").clicked() {
                    ctx.quit();
                }
            });

            ui.menu_button("Edit", |ui| {
                let _remove_dup_btn = ui.button("Remove duplicates");
            });

            ui.menu_button("Playback", |ui| {
                let play_btn = ui.button("Play");
                let stop_btn = ui.button("Stop");
                let pause_btn = ui.button("Pause");
                let next_btn = ui.button("Next");
                let prev_btn = ui.button("Previous");

                if let Some(_selected_track) = &ctx.player.as_mut().unwrap().selected_track {
                    if play_btn.clicked() {
                        ctx.player.as_mut().unwrap().play();
                    }

                    if stop_btn.clicked() {
                        ctx.player.as_mut().unwrap().stop();
                    }

                    if pause_btn.clicked() {
                        ctx.player.as_mut().unwrap().pause();
                    }

                    if next_btn.clicked() {
                        ctx.player
                            .as_mut()
                            .unwrap()
                            .next(&ctx.playlists[(ctx.current_playlist_idx).unwrap()])
                    }

                    if prev_btn.clicked() {
                        ctx.player
                            .as_mut()
                            .unwrap()
                            .previous(&ctx.playlists[(ctx.current_playlist_idx).unwrap()])
                    }
                }
            });

            ui.menu_button("Library", |ui| {
                let cfg_btn = ui.button("Configure");

                if cfg_btn.clicked() {
                    ctx.is_library_cfg_open = true;
                };
            });

            ui.menu_button("View", |ui| {
               let osc_button = ui.button("Oscilloscope");

               if osc_button.clicked() {
                   ctx.show_oscilloscope = !ctx.show_oscilloscope;
               } 
            });

            ui.menu_button("Help", |ui| {
                let _about_btn = ui.button("About");
            });

            if ctx.is_library_cfg_open {
                // TODO - Turn this library configuation into a separate component
                eframe::egui::Window::new("Library Configuration")
                    .default_width(480.0)
                    .default_height(600.0)
                    .resizable([true, true])
                    .collapsible(false)
                    .show(ui.ctx(), |ui| {
                        let available_height = ui.available_height();
                        let table = TableBuilder::new(ui)
                            .striped(true)
                            .resizable(true)
                            .cell_layout(eframe::egui::Layout::left_to_right(
                                eframe::egui::Align::Center,
                            ))
                            .column(
                                // Path
                                Column::remainder().at_least(40.0).clip(true),
                            )
                            .column(Column::remainder()) // State
                            .sense(eframe::egui::Sense::click())
                            .min_scrolled_height(0.0)
                            .max_scroll_height(available_height);

                        table
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.strong("Path");
                                });
                                header.col(|ui| {
                                    ui.strong("Status");
                                });
                            })
                            .body(|mut body| {
                                for path in ctx.library.paths().iter() {
                                    body.row(20.0, |mut row| {
                                        let row_id = path.id();
                                        row.set_selected(
                                            ctx.lib_config_selections.contains(&row_id),
                                        );
                                        row.col(|ui| {
                                            ui.label(
                                                path.path()
                                                    .clone()
                                                    .into_os_string()
                                                    .into_string()
                                                    .unwrap_or("Could not format path".to_string()),
                                            );
                                        });

                                        row.col(|ui| {
                                            ui.style_mut().wrap_mode =
                                                Some(eframe::egui::TextWrapMode::Extend);
                                            ui.label("Status unknown");
                                        });

                                        // Toggle Row Clicked Status
                                        if row.response().clicked() {
                                            if ctx.lib_config_selections.contains(&row_id) {
                                                ctx.lib_config_selections.remove(&row_id);
                                            } else {
                                                ctx.lib_config_selections.insert(row_id);
                                            }
                                        }
                                    })
                                }
                            });

                        ui.separator();

                        ui.horizontal(|ui| {
                            if ui.button("Add path").clicked() {
                                if let Some(new_path) = rfd::FileDialog::new().pick_folder() {
                                    ctx.library.add_path(new_path);
                                }
                            }

                            if ui.button("Remove selected paths").clicked() {
                                // TODO - Should only appear clickable when a path is selected.
                                // Will also remove any files in the library with the same LibraryPathId
                                if !ctx.lib_config_selections.is_empty() {
                                    for path_id in ctx.lib_config_selections.iter() {
                                        ctx.library.remove_path(*path_id);
                                    }
                                }
                            }

                            if ui.button("Cancel").clicked() {
                                ctx.is_library_cfg_open = false;
                            }

                            if ui.button("Save").clicked() {
                                for lib_path in ctx
                                    .library
                                    .paths()
                                    .iter()
                                    .filter(|p| p.status() == LibraryPathStatus::NotImported)
                                {
                                    ctx.import_library_paths(lib_path);
                                }
                                ctx.is_library_cfg_open = false;
                            }
                        })
                    });
            }
        });
    }
}
