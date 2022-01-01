use id3::Tag;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Library {
    root_path: PathBuf,
    items: Option<Vec<LibraryItem>>,
}

impl Library {
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            root_path,
            items: None,
        }
    }

    pub fn build(&mut self) {
        let mut items = vec![];

        let files = WalkDir::new(&self.root_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .skip(1)
            .filter(|entry| entry.file_type().is_file());

        for entry in files {
            let tag = Tag::read_from_path(&entry.path());

            let library_item = match tag {
                Ok(tag) => LibraryItem::new(entry.path().to_path_buf())
                    .set_title(tag.title().unwrap_or("?").to_string())
                    .set_artist(tag.artist().unwrap_or("?").to_string())
                    .set_album(tag.album().unwrap_or("?").to_string())
                    .set_year(tag.year().unwrap_or(0))
                    .set_genre(tag.genre().unwrap_or("?").to_string()),
                Err(_err) => {
                    tracing::warn!("Couldn't parse to id3: {:?}", &entry.path());
                    LibraryItem::new(entry.path().to_path_buf())
                }
            };

            items.push(library_item.clone());
        }

        self.items = Some(items);
    }

    pub fn root_path(&self) -> PathBuf {
        self.root_path.clone()
    }

    pub fn items(&self) -> Option<Vec<LibraryItem>> {
        self.items.clone()
    }
}

#[derive(Debug, Clone)]
pub struct LibraryItem {
    path: PathBuf,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    year: Option<i32>,
    genre: Option<String>,
}

impl LibraryItem {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            title: None,
            artist: None,
            album: None,
            year: None,
            genre: None,
        }
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn set_title(&mut self, title: String) -> Self {
        self.title = Some(title);
        self.to_owned()
    }

    pub fn title(&self) -> Option<String> {
        self.title.clone()
    }

    pub fn set_artist(&mut self, artist: String) -> Self {
        self.artist = Some(artist);
        self.to_owned()
    }

    pub fn artist(&self) -> Option<String> {
        self.artist.clone()
    }

    pub fn set_album(&mut self, album: String) -> Self {
        self.album = Some(album);
        self.to_owned()
    }

    pub fn album(&self) -> Option<String> {
        self.album.clone()
    }

    pub fn set_year(&mut self, year: i32) -> Self {
        self.year = Some(year);
        self.to_owned()
    }

    pub fn year(&self) -> Option<i32> {
        self.year.clone()
    }

    pub fn set_genre(&mut self, genre: String) -> Self {
        self.genre = Some(genre);
        self.to_owned()
    }

    pub fn genre(&self) -> Option<String> {
        self.genre.clone()
    }
}
