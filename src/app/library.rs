use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    paths: Vec<LibraryPath>,
    items: Vec<LibraryItem>,
    library_view: LibraryView,
}

impl Library {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            items: Vec::new(),
            library_view: LibraryView {
                view_type: ViewType::Album,
                containers: Vec::new(),
            },
        }
    }

    pub fn paths(&self) -> &Vec<LibraryPath> {
        &self.paths
    }

    pub fn add_path(&mut self, path: PathBuf) -> bool {
        if self.paths.iter().any(|p| *p.path() == path) {
            false
        } else {
            let new_path = LibraryPath::new(path);
            self.paths.push(new_path);
            true
        }
    }

    pub fn remove_path(&mut self, path_id: LibraryPathId) {
        // Remove the path from the library path list
        if let Some(idx) = self.paths.iter().position(|l| l.id() == path_id) {
            self.paths.remove(idx);
        }

        // Remove the actual items.
        while let Some(idx) = self
            .items
            .iter()
            .position(|item| item.library_id() == path_id)
        {
            self.items.swap_remove(idx);
        }

        // Remove the view container items
        for mut container in &mut self.library_view.containers {
            while let Some(ct_idx) = container
                .items
                .iter()
                .position(|ci| ci.library_id() == path_id)
            {
                container.items.swap_remove(ct_idx);
            }
        }

        // Remove the empty containers
        while let Some(idx) = self
            .library_view
            .containers
            .iter()
            .position(|ct| ct.items.is_empty())
        {
            self.library_view.containers.swap_remove(idx);
        }
    }

    pub fn set_path_to_imported(&mut self, id: LibraryPathId) {
        for path in self.paths.iter_mut() {
            if path.id() == id {
                path.set_status(LibraryPathStatus::Imported);
            }
        }
    }

    pub fn items(&self) -> &Vec<LibraryItem> {
        self.items.as_ref()
    }

    pub fn view(&self) -> &LibraryView {
        &self.library_view
    }

    pub fn add_item(&mut self, library_item: LibraryItem) {
        self.items.push(library_item);
    }

    pub fn add_view(&mut self, library_view: LibraryView) {
        let mut new = library_view.containers.clone();

        self.library_view.containers.append(&mut new);
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct LibraryPath {
    id: LibraryPathId,
    path: PathBuf,
    status: LibraryPathStatus,
}

impl LibraryPath {
    pub fn new(path: PathBuf) -> Self {
        use rand::Rng; // TODO - use ULID?
        Self {
            path,
            status: LibraryPathStatus::NotImported,
            id: LibraryPathId::new(rand::thread_rng().gen()),
        }
    }

    pub fn id(&self) -> LibraryPathId {
        self.id
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn status(&self) -> LibraryPathStatus {
        self.status
    }

    pub fn set_status(&mut self, status: LibraryPathStatus) {
        self.status = status;
    }
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct LibraryPathId(usize);

impl LibraryPathId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum LibraryPathStatus {
    NotImported,
    Imported,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LibraryItem {
    library_id: LibraryPathId,
    path: PathBuf,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    year: Option<i32>,
    genre: Option<String>,
    track_number: Option<u32>,
    key: usize,
}

impl LibraryItem {
    pub fn new(path: PathBuf, library_id: LibraryPathId) -> Self {
        use rand::Rng; // TODO - use ULID?
        Self {
            library_id,
            path,
            title: None,
            artist: None,
            album: None,
            year: None,
            genre: None,
            track_number: None,
            key: rand::thread_rng().gen(),
        }
    }

    pub fn library_id(&self) -> LibraryPathId {
        self.library_id
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn key(&self) -> usize {
        self.key
    }

    pub fn set_title(&mut self, title: Option<&str>) -> Self {
        if let Some(title) = title {
            self.title = Some(title.to_string());
        }

        self.to_owned()
    }

    pub fn title(&self) -> Option<String> {
        self.title.clone()
    }

    pub fn set_artist(&mut self, artist: Option<&str>) -> Self {
        if let Some(artist) = artist {
            self.artist = Some(artist.to_string());
        }
        self.to_owned()
    }

    pub fn artist(&self) -> Option<String> {
        self.artist.clone()
    }

    pub fn set_album(&mut self, album: Option<&str>) -> Self {
        if let Some(album) = album {
            self.album = Some(album.to_string());
        }
        self.to_owned()
    }

    pub fn album(&self) -> Option<String> {
        self.album.clone()
    }

    pub fn set_year(&mut self, year: Option<i32>) -> Self {
        self.year = year;
        self.to_owned()
    }

    pub fn year(&self) -> Option<i32> {
        self.year.clone()
    }

    pub fn set_genre(&mut self, genre: Option<&str>) -> Self {
        if let Some(genre) = genre {
            self.genre = Some(genre.to_string());
        }
        self.to_owned()
    }

    pub fn genre(&self) -> Option<String> {
        self.genre.clone()
    }

    pub fn set_track_number(&mut self, track_number: Option<u32>) -> Self {
        self.track_number = track_number;
        self.to_owned()
    }

    pub fn track_number(&self) -> Option<u32> {
        self.track_number.clone()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LibraryView {
    pub view_type: ViewType,
    pub containers: Vec<LibraryItemContainer>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LibraryItemContainer {
    pub name: String,
    pub items: Vec<LibraryItem>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ViewType {
    Album,
    Artist,
    Genre,
}
