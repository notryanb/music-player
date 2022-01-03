# Music Player

A simple GUI music player inspired by foobar2000 written in Rust using [egui](https://github.com/emilk/egui).
The goal of this project is to learn about making gui/ native apps, audio, databases / text search.
It is not meant to be used as a serious audio player.

## Goals

- Basic music player functionality. Play, pause, stop.
- Create a music library, which is indexed for searching.
- Parse id3 tags from tracks for use with indexing.
- Create playlists, which can be saved, opened, edited, reordered
- Drag n' Drop tracks from the music library into the playlist.
- Save last state of the app when closing.

## Stretch goals

- See if I can make right-click context menus.
- Visualizations
- Stream audio
- Swappable frontend so I can try other Rust cross platform gui libaries.
- Scrubbable audio. ie. Keep position in audio and arbitrarily move to any position

## Stuff to fix or implement

- [x] Reference playlists by index or actual reference (not a clone...), so info is not lost when changing playlist context
- [x] Double clicking track automatically starts to play it.
- [x] Remove playlists.
- [x] Un-named playlists get `(idx)` appended 
- [x] Playlist tab section stacks playlist tabs when they don't fit.
- [x] Add Next and Previous controls
- [x] Pause is a toggle
- [x] Play restarts the track
- [x] Add volume control slider
- [x] Implement library
- [x] Refactor so the items parsed in the library are the primary data type passed around instead of separate library items and tracks.
- [ ] Selected track is highlighted.
- [ ] Set currently playing track as app Title
- [ ] Display playlist as a table [Playing, Track #, Artist, Album Title, etc... ]
- [ ] Add player indicators next to the track
- [ ] Playlist plays to end after track is selected.
- [ ] Remove tracks from playlist.
- [ ] Reorder items in playlist.
- [ ] Save playlists.
- [ ] Save app state on close.
- [ ] Handle files which can't be decoded correctly into audio. 
- [ ] Implement library search.
- [ ] Differentiate between a selected track and the currently playing one.
- [ ] Library display options [ album, artist, year, genre, folder structure, etc...]
- [ ] Improve library build performance and probably offload to a non-ui thread.
- [ ] Library Item hashable?
- [ ] Refactor into more sensible responsibilities.
- [ ] Stop with all the cloning... seriously. Everything is cloned.