#![warn(clippy::pedantic)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::fn_params_excessive_bools)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::assigning_clones)]

use crash_reporter::CrashReporter;
use playlist_settings::AfterSong;
use playlist_settings::PersistentSettings;
use playlist_settings::SessionSettings;
use rdev::Event;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{fs, thread};
use std::fs::File;

mod crash_reporter;
mod handle_input;
mod playlist_settings;
mod utils;

fn main() {
    println!("Music player started\nType 'commands' to see available commands");

    let mut crash_reporter = CrashReporter::new();
    let paths = get_song_paths();
    setup_playlist_settings_file();

    let volume = playlist_settings::get_persistent_settings().volume;
    if (volume - 1.0).abs() > f32::EPSILON {
        println!("playlist volume: {}%", (volume * 100.0).round());
    }

    let mut session_settings = SessionSettings::default();

    let (_stream, stream_handle) =
        OutputStream::try_default().expect("Failed to get default output stream");
    let audio_player = Sink::try_new(&stream_handle).expect("Failed to create a new Sink");
    let index = get_next_song_index(&mut session_settings, &paths);
    crash_reporter.next_song(get_song_name(&paths[index]), session_settings.clone());
    let (source, _, song_name) = play_next_song(index, &paths, &mut session_settings);
    session_settings.song_duration = source
        .total_duration();
    audio_player.append(source);
    let mut persistent_settings = playlist_settings::get_persistent_settings();
    let song_settings = persistent_settings.get_song_settings(&song_name);
    let track_volume = song_settings.song_volume * volume;
    audio_player.set_volume(track_volume);
    persistent_settings.set_song_probability(&song_name, 0);
    persistent_settings.accumulate_play_count(&song_name);
    playlist_settings::update_settings(&persistent_settings);

    crash_reporter.set_session_settings(session_settings.clone());

    let new_messages = Arc::new(Mutex::new(Vec::new()));
    let new_messages_clone = Arc::clone(&new_messages);

    thread::spawn(move || loop {
        let input = utils::get_console_input();
        new_messages_clone.lock().unwrap().push(input);
    });

    let new_key_events = Arc::new(Mutex::new(Vec::new()));
    let new_key_events_clone = Arc::clone(&new_key_events);

    thread::spawn(move || {
        rdev::listen(move |event| {
            new_key_events_clone.lock().unwrap().push(event);
        })
        .expect("Failed to listen for keyboard events");
    });

    loop {
        crash_reporter.set_session_settings(session_settings.clone());

        check_new_commands(
            &new_messages,
            &audio_player,
            &mut session_settings,
            &paths,
            &mut crash_reporter,
        );

        check_new_key_events(
            &new_key_events,
            &audio_player,
            &mut session_settings,
            &paths,
            &mut crash_reporter,
        );

        if audio_player.empty() {
            audio_player.clear();
            let index = get_next_song_index(&mut session_settings, &paths);
            crash_reporter.next_song(get_song_name(&paths[index]), session_settings.clone());
            let (source, _, song_name) = match session_settings.after_song {
                AfterSong::PlaySong(next_song) => {
                    let song = index_song(&paths, next_song);
                    (song.0, next_song, song.1)
                }
                _ => play_next_song(index, &paths, &mut session_settings),
            };
            audio_player.append(source);
            let song_settings =
                playlist_settings::get_persistent_settings().get_song_settings(&song_name);
            audio_player.set_volume(
                session_settings.playback_playlist_volume() * song_settings.song_volume,
            );
            audio_player.play();
            if let AfterSong::Pause = session_settings.after_song {
                audio_player.pause();
                println!("paused");
            }
            session_settings.after_song = AfterSong::Continue;
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
    index: usize,
    paths: &[PathBuf],
    session_settings: &mut SessionSettings,
) -> (Decoder<BufReader<File>>, usize, String) {
    let mut persistent_settings = playlist_settings::get_persistent_settings();
    if session_settings.shuffle {
        for i in 0..paths.len() {
            persistent_settings.set_song_probability(paths[i].to_str().expect("path has no name"), persistent_settings.get_probability_distribution(paths)[i] + 1);
        }
        persistent_settings.set_song_probability(paths[index].to_str().expect("path has no name"), 0);
    }
    persistent_settings.accumulate_play_count(paths[index].to_str().expect("path has no name"));
    playlist_settings::update_settings(&persistent_settings);
    let (source, file_name) = index_song(paths, index);
    session_settings.current_song_index = index;
    session_settings.current_song_name = file_name.clone();
    session_settings.duration_start = Instant::now();
    session_settings.song_duration = source
        .total_duration();
    session_settings.reset_song_progress();
    println!(
        "Now playing: {} ({})",
        session_settings.current_song_name,
        session_settings.format_song_duration(),
    );
    let song_settings = persistent_settings.get_song_settings(&file_name);
    let song_volume = song_settings.song_volume;
    let playlist_volume = persistent_settings.volume;
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

fn get_next_song_index(session_settings: &mut SessionSettings, paths: &[PathBuf]) -> usize {
    let settings = playlist_settings::get_persistent_settings();
    if session_settings.shuffle {
        let mut modified_song_probability_distribution = Vec::new();
        for i in 0..paths.len() {
            let song_settings = settings.get_song_settings(&get_song_name(&paths[i]));
            if session_settings.exclude_lyrics && song_settings.has_lyrics {
                modified_song_probability_distribution.push(0);
                continue;
            }
            let p = settings.get_probability_distribution(paths)[i];
            let star_factor = if song_settings.starred { 2 } else { 1 };
            modified_song_probability_distribution.push(p * star_factor);
        }
        utils::weighted_random_selection(
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
    session_settings: &mut SessionSettings,
    paths: &[PathBuf],
    crash_reporter: &mut CrashReporter,
) {
    let mut messages = Vec::new();
    let mut new_messages = new_messages.lock().unwrap();
    messages.append(&mut *new_messages);
    for message in messages {
        crash_reporter.upcoming_command(message.clone(), session_settings.clone());
        handle_input::handle_console_commands(
            message.trim(),
            audio_player,
            session_settings,
            paths,
            crash_reporter,
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
    session_settings: &mut SessionSettings,
    paths: &[PathBuf],
    crash_reporter: &mut CrashReporter,
) {
    let mut key_events = Vec::new();
    let mut new_key_events = new_key_events.lock().unwrap();
    key_events.append(&mut *new_key_events);
    for key_event in key_events {
        crash_reporter.upcoming_key_event(key_event.clone(), session_settings.clone());
        handle_input::handle_key_event(
            &key_event,
            audio_player,
            session_settings,
            paths,
            crash_reporter,
        );
    }
}
