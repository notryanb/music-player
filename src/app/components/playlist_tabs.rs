use super::AppComponent;
use crate::app::App;
use eframe::egui;

pub struct PlaylistTabs;

impl AppComponent for PlaylistTabs {
    type Context = App;

    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for (idx, playlist) in ctx.playlists.iter().enumerate() {
                let playlist_tab = ui.add(
                    egui::Label::new(playlist.get_name().unwrap()).sense(egui::Sense::click()),
                );

                if playlist_tab.clicked() {
                    ctx.current_playlist_idx = Some(idx);
                }

                // TODO - make this bring up a context menu, however just delete for
                // now.
                if playlist_tab.clicked_by(egui::PointerButton::Secondary) {
                    ctx.playlist_idx_to_remove = Some(idx);
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
