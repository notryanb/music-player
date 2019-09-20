extern crate crossterm;
extern crate rodio;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crossterm::{AsyncReader, ClearType, Color, Colorize, Crossterm, InputEvent, KeyEvent, RawScreen};
use rodio::{Device, Sink};

pub enum TrackState {
    Unstarted,
    Stopped,
    Playing,
    Paused,
}

pub struct Player<'a> {
    pub track_state: TrackState,
    pub track_path: &'a Path,
    pub sink: Sink,
    pub device: Device,
}

impl Player<'_> {
    pub fn new<'a>(track_path: &'a str) -> Player {
        let device = rodio::default_output_device().unwrap();

        Player {
            track_path: Path::new(track_path),
            track_state: TrackState::Unstarted,
            sink: Sink::new(&device),
            device: rodio::default_output_device().unwrap(),
        }
    }

    pub fn start(&mut self) {
        let file = File::open(self.track_path).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).expect("Could not create decoder from file");

        self.sink = Sink::new(&self.device);

        self.sink.append(source);
        self.track_state = TrackState::Playing;
    }

    pub fn stop(&mut self) {
        self.sink.stop();
        self.track_state = TrackState::Stopped
    }

    pub fn play(&mut self) {
        self.sink.play();
        self.track_state = TrackState::Playing
    }

    pub fn pause(&mut self) {
        self.sink.pause();
        self.track_state = TrackState::Paused
    }
}



fn main() {
    let raw_mode = RawScreen::into_raw_mode();
    let crossterm = Crossterm::new();

    crossterm.cursor().hide();
    let mut stdin = crossterm.input().read_async();
    let mut player = Player::new("E:\\Mp3s\\Fuel\\Fuel - Monuments to Excess\\fuel-03-some gods.mp3");

    loop {
        let pressed_key = stdin.next();


        if let Some(InputEvent::Keyboard(KeyEvent::Char(character))) = pressed_key {
            match character {
                'e' =>  { 
                    println!("Pressed e(xit) ... quitting");
                },
                's' => {
                    println!("Pressed s"); // stop
                    match &player.track_state {
                        TrackState::Unstarted | TrackState::Stopped => player.start(),
                        TrackState::Playing | TrackState::Paused => player.stop(),
                    }
                },
                'p' =>{
                    println!("Pressed p"); // play
                    match &player.track_state {
                        TrackState::Paused => player.play(),
                        TrackState::Playing => player.pause(),
                        TrackState::Stopped | TrackState::Unstarted => println!("can't pause or play a track that is stopped or not started"),
                    }
                }, // play
                _ => ()
            }
        }
    }
}


// fn main() {
//     use std::{thread, time};


//     let track_path = "E:\\Mp3s\\45 grave\\45 grave - concerned citizen.mp3";
//     let file = File::open(track_path).unwrap();
//     let source = rodio::Decoder::new(BufReader::new(file)).expect("Could not create decoder from file");
//     let device = rodio::default_output_device().unwrap();
//     let sink = Sink::new(&device);

//     println!("Sink Len [before starting first track]: {:?}", sink.len());
//     println!("Sink Empty [before starting first track]: {:?}", sink.empty());

//     sink.append(source);

//     println!("Sink Len [after starting first track]: {:?}", sink.len());
//     println!("Sink Empty [after starting first track]: {:?}", sink.empty());
//     println!("Sleeping 10 seconds...");

//     let five_seconds = time::Duration::from_millis(5_000);
//     thread::sleep(five_seconds);

//     println!("stopping sink");
//     sink.stop();

//     let five_seconds = time::Duration::from_millis(5_000);
//     thread::sleep(five_seconds);

//     println!("Sink Len [after stopping first track]: {:?}", sink.len());
//     println!("Sink Empty [after stopping first track]: {:?}", sink.empty());
//     println!("appending same newly opened file and decoder");

//     let track_path2 = "E:\\Mp3s\\45 grave\\45 grave - concerned citizen.mp3";
//     let file2 = File::open(track_path2).unwrap();
//     let source2 = rodio::Decoder::new(BufReader::new(file2)).expect("Could not create decoder from file");
//     let sink2 = Sink::new(&device);
//     sink2.append(source2);
//     // sink.play();

//     println!("Sink Len [after starting second track]: {:?}", sink2.len());
//     println!("Sink Empty [after second track]: {:?}", sink2.empty());

//     let five_seconds = time::Duration::from_millis(5_000);
//     thread::sleep(five_seconds);
// }