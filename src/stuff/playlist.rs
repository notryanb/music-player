use std::path::PathBuf;

// TODO - Stop all the clonin'!
#[derive(Debug, Clone)]
pub struct Playlist {
    name: Option<String>,
    pub tracks: Vec<Track>,
    pub selected: Option<Track>,
}

// TODO impl a builder pattern?
impl Playlist {
    pub fn new() -> Self {
        Self {
            name: None,
            tracks: vec![],
            selected: None,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    pub fn get_name(&self) -> Option<String> {
        self.name.clone()
    }

    pub fn add(&mut self, track: Track) {
        self.tracks.push(track);
    }

    // TODO - should probably return a Result
    pub fn remove(&mut self, idx: usize) {
        self.tracks.remove(idx);
    }

    // TODO - should probably return a Result
    pub fn reorder(&mut self, current_pos: usize, destination_pos: usize) {
        let track = self.tracks.remove(current_pos);
        self.tracks.insert(destination_pos, track);
    }

    // TODO - should probably return a Result
    pub fn select(&mut self, idx: usize) {
        self.selected = Some(self.tracks[idx].clone());
    }

    pub fn get_pos(&self, track: &Track) -> Option<usize> {
        self.tracks.iter().position(|t| t == track)
    }
}

// TODO - Probably shouldn't hold the actual bytes, but borrowed tag information after the file is opened.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Track {
    pub path: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_playlist() {
        let playlist = Playlist::new();

        assert_eq!(playlist.name, None);
        assert_eq!(playlist.tracks.len(), 0);
        assert_eq!(playlist.selected, None);
    }

    #[test]
    fn set_name() {
        let mut playlist = Playlist::new();
        playlist.set_name("Test".to_string());

        assert_eq!(playlist.name, playlist.get_name());
        assert_eq!(playlist.tracks.len(), 0);
        assert_eq!(playlist.selected, None);
    }

    #[test]
    fn add_track_to_playlist() {
        let track = Track {
            path: PathBuf::from(r"C:\music\song.mp3"),
        };

        let mut playlist = Playlist::new();
        playlist.add(track);

        assert_eq!(playlist.tracks.len(), 1);
    }

    #[test]
    fn remove_track_from_playlist() {
        let path1 = PathBuf::from(r"C:\music\song1.mp3");
        let path2 = PathBuf::from(r"C:\music\song2.mp3");
        let path3 = PathBuf::from(r"C:\music\song3.mp3");

        let mut playlist = Playlist {
            name: Some("test".to_string()),
            tracks: vec![
                Track {
                    path: path1.clone(),
                },
                Track {
                    path: path2.clone(),
                },
                Track {
                    path: path3.clone(),
                },
            ],
            selected: None,
        };

        assert_eq!(playlist.tracks.len(), 3);

        playlist.remove(1);

        assert_eq!(playlist.tracks.len(), 2);
        assert_eq!(playlist.tracks.first().unwrap().path, path1);
        assert_eq!(playlist.tracks.last().unwrap().path, path3);
    }

    #[test]
    fn reorder_track_in_playlist() {
        let path1 = PathBuf::from(r"C:\music\song1.mp3");
        let path2 = PathBuf::from(r"C:\music\song2.mp3");
        let path3 = PathBuf::from(r"C:\music\song3.mp3");

        let mut playlist = Playlist {
            name: Some("test".to_string()),
            tracks: vec![
                Track {
                    path: path1.clone(),
                },
                Track {
                    path: path2.clone(),
                },
                Track {
                    path: path3.clone(),
                },
            ],
            selected: None,
        };

        assert_eq!(playlist.tracks.len(), 3);

        playlist.reorder(0, 2);

        assert_eq!(playlist.tracks.len(), 3);
        assert_eq!(playlist.tracks[0].path, path2);
        assert_eq!(playlist.tracks[1].path, path3);
        assert_eq!(playlist.tracks[2].path, path1);
    }

    #[test]
    fn select_track() {
        let track1 = Track {
            path: PathBuf::from(r"C:\music\song1.mp3"),
        };
        let track2 = Track {
            path: PathBuf::from(r"C:\music\song2.mp3"),
        };
        let track3 = Track {
            path: PathBuf::from(r"C:\music\song3.mp3"),
        };

        let mut playlist = Playlist {
            name: Some("test".to_string()),
            tracks: vec![track1, track2, track3.clone()],
            selected: None,
        };

        assert_eq!(playlist.tracks.len(), 3);

        playlist.select(2);

        assert_eq!(playlist.selected, Some(track3));
    }
}
