use super::AppComponent;
use crate::app::{App, Library, LibraryItem};
use itertools::Itertools;

use id3::Tag;
use rayon::prelude::*;

pub struct LibraryComponent;

impl AppComponent for LibraryComponent {
    type Context = App;

    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui) {
        eframe::egui::ScrollArea::both().show(ui, |ui| {
            if ui.button("Add Library path").clicked() {
                if let Some(lib_path) = rfd::FileDialog::new().pick_folder() {
                    tracing::info!("adding library path...");
                    ctx.library = Some(Library::new(lib_path.clone()));

                    let tx = ctx.library_sender.as_ref().unwrap().clone();
                    std::thread::spawn(move || {
                        let files: Vec<walkdir::DirEntry> = walkdir::WalkDir::new(&lib_path)
                            .into_iter()
                            .filter_map(|e| e.ok())
                            .skip(1)
                            .filter(|entry| entry.file_type().is_file())
                            .collect();

                        let items = files
                            .par_iter()
                            .map(|entry| {
                                let tag = Tag::read_from_path(&entry.path());

                                let library_item = match tag {
                                    Ok(tag) => LibraryItem::new(entry.path().to_path_buf())
                                        .set_title(tag.title())
                                        .set_artist(tag.artist())
                                        .set_album(tag.album())
                                        .set_year(tag.year())
                                        .set_genre(tag.genre())
                                        .set_track_number(tag.track()),
                                    Err(_err) => {
                                        tracing::warn!(
                                            "Couldn't parse to id3: {:?}",
                                            &entry.path()
                                        );
                                        LibraryItem::new(entry.path().to_path_buf())
                                    }
                                };

                                return library_item;
                            })
                            .collect::<Vec<LibraryItem>>();

                        tx.send(items).expect("Failed to send")
                    });
                }
            }

            if let Some(library) = &ctx.library {
                let root_path_string = &library
                    .root_path()
                    .clone()
                    .into_os_string()
                    .into_string()
                    .unwrap();

                eframe::egui::CollapsingHeader::new(eframe::egui::RichText::new(root_path_string))
                    .default_open(true)
                    .show(ui, |ui| {
                        let mut library_items_clone = library.items().clone();

                        // In order for group by to work from itertools, items must be consecutive, so sort them first.
                        library_items_clone.sort_by_key(|item| item.album());

                        for (key, group) in &library_items_clone
                            .into_iter()
                            .group_by(|item| item.album().unwrap_or("?".to_string()).to_string())
                        {
                            let items =
                                group.map(|item| item.clone()).collect::<Vec<LibraryItem>>();

                            let library_group = eframe::egui::CollapsingHeader::new(
                                eframe::egui::RichText::new(key),
                            )
                            .default_open(false)
                            .selectable(true)
                            .show(ui, |ui| {
                                for item in &items {
                                    let item_label = ui.add(
                                        eframe::egui::Label::new(eframe::egui::RichText::new(
                                            item.title().unwrap_or("?".to_string()),
                                        ))
                                        .sense(eframe::egui::Sense::click()),
                                    );

                                    if item_label.double_clicked() {
                                        if let Some(current_playlist_idx) =
                                            &ctx.current_playlist_idx
                                        {
                                            let current_playlist =
                                                &mut ctx.playlists[*current_playlist_idx];

                                            current_playlist.add(item.clone());
                                        }
                                    }
                                }
                            });

                            if let Some(current_playlist_idx) = &ctx.current_playlist_idx {
                                let current_playlist = &mut ctx.playlists[*current_playlist_idx];

                                if library_group.header_response.double_clicked() {
                                    for item in items {
                                        current_playlist.add(item.clone());
                                    }
                                }
                            }
                        }
                    });
            }
        });
    }
}
