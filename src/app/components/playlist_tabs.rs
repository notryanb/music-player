use super::AppComponent;
use crate::app::App;
use eframe::egui;

pub struct PlaylistTabs;

impl AppComponent for PlaylistTabs {
    type Context = App;

    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for (idx, playlist) in ctx.playlists.iter_mut().enumerate() {
                let mut playlist_name = playlist.get_name().unwrap();

                if playlist.is_editing_name {
                    let response = ui.add(egui::TextEdit::singleline(&mut playlist_name));

                    if response.changed() {
                        playlist.set_name(playlist_name);
                    }

                    if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        playlist.is_editing_name = false;
                    }
                } else {
                    let playlist_tab = ui.add(
                        egui::Label::new(playlist_name).sense(egui::Sense::click())
                    );

                    if playlist_tab.clicked() {
                        ctx.current_playlist_idx = Some(idx);
                    }

                    egui::containers::Popup::context_menu(&playlist_tab).id(egui::Id::new("playlist_options_menu"))
                        .show(|ui| {
                            if ui.button("Remove Playlist").clicked() {
                                ctx.playlist_idx_to_remove = Some(idx);
                            }
                        });

                    if playlist_tab.double_clicked() {
                        playlist.is_editing_name = true;
                    }
                }
            }


            if let Some(idx) = ctx.playlist_idx_to_remove {
                ctx.playlist_idx_to_remove = None;

                // Because the current playlist is referenced via index, we need to take
                // into account that the index may be out of bounds when removing a
                // playlist. This should be resolved when I figure out how to reference the
                // actual selected playlist.
                if let Some(mut current_playlist_idx) = ctx.current_playlist_idx {
                    if current_playlist_idx == 0 && idx == 0 {
                        ctx.current_playlist_idx = None;
                    } else if current_playlist_idx >= idx {
                        current_playlist_idx -= 1;
                        ctx.current_playlist_idx = Some(current_playlist_idx);
                    }
                }

                ctx.playlists.remove(idx);
            }
        });
    }
}
