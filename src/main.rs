#![warn(clippy::pedantic)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::fn_params_excessive_bools)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::assigning_clones)]

use playlist_settings::AfterSong;
use rand::rngs::ThreadRng;
use rand::Rng;
use rdev::Event;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::{fs::File, io};

use crate::playlist_settings::PersistentSettings;
use crate::playlist_settings::SessionSettings;

mod handle_input;
mod playlist_settings;

fn main() {
    println!("Music player started\nType 'commands' to see available commands");

    let paths = get_song_paths();
    setup_playlist_settings_file();

    let random = rand::thread_rng();
    let song_probability_distribution = get_default_distribution(paths.len());
    let volume = playlist_settings::get_persistent_settings().volume;

    if (volume - 1.0).abs() > f32::EPSILON {
        println!("playlist volume: {}%", (volume * 100.0).round());
    }

    let mut session_settings = SessionSettings::new(
        false,
        true,
        true,
        false,
        0,
        String::new(),
        song_probability_distribution,
        Instant::now(),
        Duration::ZERO,
        Duration::ZERO,
        AfterSong::Continue,
        random,
    );

    let (_stream, stream_handle) =
        OutputStream::try_default().expect("Failed to get default output stream");
    let audio_player = Sink::try_new(&stream_handle).expect("Failed to create a new Sink");
    let (source, song_index, song_name) = play_next_song(&paths, &mut session_settings);
    session_settings.song_duration = source
        .total_duration()
        .expect("Failed to get song duration");
    audio_player.append(source);
    let song_settings = playlist_settings::get_persistent_settings().get_song_settings(&song_name);
    let track_volume = song_settings.song_volume * volume;
    audio_player.set_volume(track_volume);
    session_settings.song_probability_distribution[song_index] = 0;

    let new_messages = Arc::new(Mutex::new(Vec::new()));
    let new_messages_clone = Arc::clone(&new_messages);

    thread::spawn(move || loop {
        let mut input_buffer = String::new();
        io::stdin()
            .read_line(&mut input_buffer)
            .expect("Failed to read input");
        new_messages_clone.lock().unwrap().push(input_buffer);
    });

    let new_key_events = Arc::new(Mutex::new(Vec::new()));
    let new_key_events_clone = Arc::clone(&new_key_events);

    thread::spawn(move || {
        rdev::listen(move |event| {
            if session_settings.key_events_enabled {
                new_key_events_clone.lock().unwrap().push(event);
            }
        })
        .expect("Failed to listen for keyboard events");
    });

    loop {
        check_new_commands(&new_messages, &audio_player, &mut session_settings, &paths);

        check_new_key_events(
            &new_key_events,
            &audio_player,
            &mut session_settings,
            &paths,
        );

        if audio_player.empty() {
            audio_player.clear();
            let (source, _, song_name) = match session_settings.after_song {
                AfterSong::PlaySong(next_song) => {
                    let song = index_song(&paths, next_song);
                    (song.0, next_song, song.1)
                }
                _ => play_next_song(&paths, &mut session_settings),
            };
            audio_player.append(source);
            let song_settings =
                playlist_settings::get_persistent_settings().get_song_settings(&song_name);
            audio_player.set_volume(session_settings.playback_volume() * song_settings.song_volume);
            audio_player.play();
            if let AfterSong::Pause = session_settings.after_song {
                audio_player.pause();
                println!("paused");
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

fn setup_playlist_settings_file() {
    let playlist_settings_path = Path::new("playlist-settings.json");
    if !playlist_settings_path.exists() {
        let json = playlist_settings::to_json(&PersistentSettings::default());
        let mut file =
            File::create(playlist_settings_path).expect("Failed to create playlist settings file");
        file.write_all(json.as_bytes())
            .expect("Failed to write playlist settings file");
    }
}

fn get_default_distribution(len: usize) -> Vec<u32> {
    let mut song_probability_distribution = Vec::new();
    song_probability_distribution.reserve_exact(len);
    song_probability_distribution.resize(len, 1);
    song_probability_distribution
}

fn get_song_paths() -> Vec<PathBuf> {
    fs::read_dir("playlist")
        .expect("Failed to find \"playlist\" directory. Please create a folder called \"playlist\" in the \"music-player\" directory.")
        .map(|path| {
            path.expect("Failed to read paths in \"playlist\" directory")
                .path()
        })
        //the first path in the playlist is reserved for a subfolder to synchronize the playlist to google drive
        .skip(1)
        .collect::<Vec<PathBuf>>()
}

fn play_next_song(
    paths: &[PathBuf],
    session_settings: &mut SessionSettings,
) -> (Decoder<BufReader<File>>, usize, String) {
    let settings = playlist_settings::get_persistent_settings();
    let index = if session_settings.shuffle {
        let mut modified_song_probability_distribution = Vec::new();
        for i in 0..paths.len() {
            let song_settings = settings.get_song_settings(&get_song_name(&paths[i]));
            if session_settings.exclude_lyrics && song_settings.has_lyrics {
                modified_song_probability_distribution.push(0);
                continue;
            }
            let p = session_settings.song_probability_distribution[i];
            let star_factor = if song_settings.starred { 2 } else { 1 };
            modified_song_probability_distribution.push(p * star_factor);
        }
        weighted_random_choice(
            &modified_song_probability_distribution,
            &mut session_settings.random,
        )
    } else {
        let mut next_song = None;
        for i in 0..paths.len() {
            let i = (session_settings.current_song_index + i + 1) % paths.len();
            if session_settings.exclude_lyrics
                && settings
                    .get_song_settings(&get_song_name(&paths[i]))
                    .has_lyrics
            {
                continue;
            }
            next_song = Some(i);
            break;
        }
        next_song.expect(
            "exclude lyrics mode is enabled but all songs in the playlist are set to have lyrics",
        )
    };
    if session_settings.shuffle {
        for i in 0..paths.len() {
            session_settings.song_probability_distribution[i] += 1;
        }
        session_settings.song_probability_distribution[index] = 0;
    }
    let (source, file_name) = index_song(paths, index);
    session_settings.current_song_index = index;
    session_settings.current_song_name = file_name.clone();
    session_settings.song_start = Instant::now();
    session_settings.song_duration = source
        .total_duration()
        .expect("Failed to get song duration");
    println!(
        "Now playing: {} ({})",
        session_settings.current_song_name,
        format_duration(&session_settings.song_duration)
    );
    let song_settings = playlist_settings::get_persistent_settings().get_song_settings(&file_name);
    let song_volume = song_settings.song_volume;
    let playlist_volume = playlist_settings::get_persistent_settings().volume;
    if (song_volume - 0.5).abs() > f32::EPSILON {
        println!(
            "Song volume: {}% (playing at {}% volume)",
            (song_volume * 100.0).round(),
            (song_volume * playlist_volume * 100.0).round()
        );
    }
    if song_settings.starred {
        println!("This song is starred");
    }
    (source, index, file_name)
}

fn format_duration(duration: &Duration) -> String {
    if duration.as_secs() % 60 < 10 {
        format!("{}:0{}", duration.as_secs() / 60, duration.as_secs() % 60)
    } else {
        format!("{}:{}", duration.as_secs() / 60, duration.as_secs() % 60)
    }
}

fn weighted_random_choice(song_probability_distribution: &[u32], random: &mut ThreadRng) -> usize {
    let mut sum = song_probability_distribution.iter().sum::<u32>();
    for (i, v) in song_probability_distribution.iter().enumerate() {
        if random.gen_bool(*v as f64 / sum as f64) {
            return i;
        }
        sum -= v;
    }
    if song_probability_distribution.is_empty() {
        panic!("there are no songs in the playlist")
    } else {
        panic!(
            "exclude lyrics mode is enabled but all songs in the playlist are set to have lyrics"
        )
    }
}

fn index_song(paths: &[PathBuf], index: usize) -> (Decoder<BufReader<File>>, String) {
    let path = &paths[index];
    let file_name = get_song_name(path);
    let file = File::open(path).expect("File does not exist in the specified directory");
    let reader = BufReader::new(file);
    let source = Decoder::new(reader).expect("Failed to decode the MP3 file");
    (source, file_name)
}

fn check_new_commands(
    new_messages: &Arc<Mutex<Vec<String>>>,
    audio_player: &Sink,
    playlist_properties: &mut SessionSettings,
    paths: &[PathBuf],
) {
    let mut messages = Vec::new();
    let mut new_messages = new_messages.lock().unwrap();
    messages.append(&mut *new_messages);
    for message in messages {
        handle_input::handle_console_commands(
            message.trim(),
            audio_player,
            playlist_properties,
            paths,
        );
    }
}

fn get_song_name(path: &Path) -> String {
    path.file_name()
        .expect("Invalid file path")
        .to_str()
        .expect("Invalid file name")
        .split_once(".mp3")
        .expect("songs should have the mp3 format")
        .0
        .replace('-', " ")
}

fn check_new_key_events(
    new_key_events: &Arc<Mutex<Vec<Event>>>,
    audio_player: &Sink,
    playlist_properties: &mut SessionSettings,
    paths: &[PathBuf],
) {
    let mut key_events = Vec::new();
    let mut new_key_events = new_key_events.lock().unwrap();
    key_events.append(&mut *new_key_events);
    for key_event in key_events {
        handle_input::handle_key_event(&key_event, audio_player, playlist_properties, paths);
    }
}
