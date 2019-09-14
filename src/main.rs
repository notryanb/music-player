extern crate crossterm;
extern crate rodio;

use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crossterm::{AsyncReader, ClearType, Color, Colorize, Crossterm, InputEvent, KeyEvent, RawScreen};
use rodio::{Decoder, Device, Sink, Source};

pub enum TrackState {
    Unstarted,
    Stopped,
    Playing,
    Paused,
}

pub struct Player<'a> {
    pub track_state: TrackState,
    pub track_path: &'a Path,
}

impl Player<'_> {
    pub fn new<'a>(track_path: &'a str) -> Player {
        Player {
            track_path: Path::new(track_path),
            track_state: TrackState::Unstarted,
        }
    }

    pub fn start(&mut self, sink: &Sink) {
        let file = File::open(self.track_path).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).expect("Could not create decoder from file");
        sink.append(source);
        println!("Sink Len [start]: {:?}", sink.len());
        self.track_state = TrackState::Playing
    }

    pub fn stop(&mut self, sink: &Sink) {
        sink.stop();
        println!("Sink Len [stop]: {:?}", sink.len());
        self.track_state = TrackState::Stopped
    }

    pub fn play(&mut self, sink: &Sink) {
        sink.play();
        println!("Sink Len [play]: {:?}", sink.len());
        self.track_state = TrackState::Playing
    }

    pub fn pause(&mut self, sink: &Sink) {
        sink.pause();
        println!("Sink Len [pause]: {:?}", sink.len());
        self.track_state = TrackState::Paused
    }
}

fn main() {
    let raw_mode = RawScreen::into_raw_mode();
    let crossterm = Crossterm::new();

    crossterm.cursor().hide();
    let mut stdin = crossterm.input().read_async();

    let device = rodio::default_output_device().unwrap();
    let sink = Sink::new(&device);

    // let file = File::open("E:\\Mp3s\\Fuel\\Fuel - Monuments to Excess\\fuel-03-some gods.mp3").unwrap();
    // let source = rodio::Decoder::new(BufReader::new(file)).expect("Could not create decoder from file");
    // sink.append(source); 

    let mut player = Player::new("E:\\Mp3s\\Fuel\\Fuel - Monuments to Excess\\fuel-03-some gods.mp3");

    loop {
        let pressed_key = stdin.next();


        if let Some(InputEvent::Keyboard(KeyEvent::Char(character))) = pressed_key {
            match character {
                'e' =>  { // exit
                    println!("Pressed e");
                    
                    // sink.append(source); 
                },
                's' => {
                    println!("Pressed s"); // stop
                    match &player.track_state {
                        TrackState::Unstarted => player.start(&sink),
                        TrackState::Playing | TrackState::Paused => player.stop(&sink),
                        TrackState::Stopped => player.start(&sink),
                    }
                },
                'p' =>{
                    println!("Pressed p"); // play
                    match &player.track_state {
                        TrackState::Paused => player.play(&sink),
                        TrackState::Playing => player.pause(&sink),
                        TrackState::Stopped | TrackState::Unstarted => println!("can't pause or play a track that is stopped or not started"),
                    }
                }, // play
                _ => ()
            }
        }
    }
}