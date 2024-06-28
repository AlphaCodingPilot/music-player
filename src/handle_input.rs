use std::{fs, path::PathBuf, process};

use rdev::{Event, EventType, Key};
use rodio::Sink;

use crate::playlist_settings::{self, SessionSettings};

pub fn handle_command(
    input_buffer: &str,
    audio_player: &Sink,
    session_settings: &mut SessionSettings,
    paths: &[PathBuf],
) {
    let input = input_buffer.trim().to_lowercase();
    let input = input.replace(' ', "");
    match input.as_str() {
        "" => (),
        "pause" => {
            if !audio_player.is_paused() {
                audio_player.pause();
                println!("paused");
            }
        }
        "resume" | "r" | "play" => {
            if audio_player.is_paused() {
                audio_player.play();
                println!("resumed");
            }
        }
        "p" | "k" => {
            pause_or_play(audio_player);
        }
        "mute" => {
            if !session_settings.is_muted {
                audio_player.set_volume(0.0);
                session_settings.is_muted = true;
                println!("muted");
            }
        }
        "unmute" => {
            if session_settings.is_muted {
                let song_volume = playlist_settings::get_persistent_settings()
                    .get_song_settings(&session_settings.current_song_name)
                    .song_volume;
                let volume = playlist_settings::get_persistent_settings().volume;
                audio_player.set_volume(volume * song_volume);
                session_settings.is_muted = false;
                println!("unmuted");
                println!("playlist volume: {}%", (volume * 100.0).round());
            }
        }
        "m" => {
            mute_or_unmute(session_settings, audio_player);
        }
        "volume+" | "v+" | "+" => {
            increase_volume(session_settings, audio_player);
        }
        "volume- " | "v-" | "-" => {
            decrease_volume(session_settings, audio_player);
        }
        "songvolume+" | "sv+" => {
            let mut settings = playlist_settings::get_persistent_settings()
                .get_song_settings(&session_settings.current_song_name);
            settings.song_volume += 0.1;
            if settings.song_volume > 1.0 {
                settings.song_volume = 1.0;
            }
            let playlist_volume = playlist_settings::get_persistent_settings().volume;
            audio_player.set_volume(settings.song_volume * playlist_volume);
            session_settings.is_muted = false;
            println!("song volume: {}%", (settings.song_volume * 100.0).round());
            playlist_settings::update_song_settings(
                session_settings.current_song_name.clone(),
                settings,
            );
        }
        "songvolume-" | "sv-" => {
            let mut settings = playlist_settings::get_persistent_settings()
                .get_song_settings(&session_settings.current_song_name);
            settings.song_volume -= 0.1;
            if settings.song_volume < 0.0 {
                settings.song_volume = 0.0;
            }
            let playlist_volume = playlist_settings::get_persistent_settings().volume;
            audio_player.set_volume(settings.song_volume * playlist_volume);
            println!("song volume: {}%", (settings.song_volume * 100.0).round());
            playlist_settings::update_song_settings(
                session_settings.current_song_name.clone(),
                settings,
            );
        }
        "volume" | "v" => {
            let volume = playlist_settings::get_persistent_settings().volume;
            let song_volume = playlist_settings::get_persistent_settings()
                .get_song_settings(&session_settings.current_song_name)
                .song_volume;
            println!("playlist volume: {}%", (volume * 100.0).round());
            println!("song volume: {}%", (song_volume * 100.0).round());
            println!(
                "combined volume: {}%",
                (volume * song_volume * 100.0).round()
            );
            if session_settings.is_muted {
                println!("The playlist is muted!");
            }
        }
        "nextsong" | "next" | "n" | "skip" => {
            next_song(audio_player, paths, session_settings);
        }
        "restartsong" | "restart" | "rs" => {
            restart_song(audio_player, paths, session_settings);
        }
        "playlist" => {
            let list = paths
                .iter()
                .map(|path| crate::get_song_name(path))
                .enumerate()
                .map(|(i, song)| {
                    let spaces = "Index".len() - i.to_string().len();
                    format!("{}{i} - {song}", " ".repeat(spaces))
                })
                .collect::<Vec<String>>()
                .join("\n");
            println!("Index - Song\n{list}\n");
        }
        "enablekeyboard" | "ekb" => {
            if !session_settings.key_events_enabled {
                session_settings.key_events_enabled = true;
                println!("keyboard shortcuts enabled");
            }
        }
        "disablekeyboard" | "dkb" => {
            if !session_settings.key_events_enabled {
                session_settings.key_events_enabled = false;
                println!("keyboard shortcuts disabled");
            }
        }
        "keyboard" | "kb" | "ks" => {
            if session_settings.key_events_enabled {
                session_settings.key_events_enabled = false;
                println!("keyboard shortcuts disabled");
            } else {
                session_settings.key_events_enabled = true;
                println!("keyboard shortcuts enabled");
            }
        }
        "enableshuffle" | "es" => {
            if !session_settings.shuffle {
                session_settings.shuffle = true;
                println!("shuffle playlist enabled");
            }
        }
        "disableshuffle" | "ds" => {
            if !session_settings.shuffle {
                session_settings.shuffle = false;
                println!("shuffle playlist disabled");
            }
        }
        "shuffle" | "sh" => {
            if session_settings.shuffle {
                session_settings.shuffle = false;
                println!("shuffle playlist disabled");
            } else {
                session_settings.shuffle = true;
                println!("shuffle playlist enabled");
            }
        }
        "star" | "s" => {
            let mut settings = playlist_settings::get_persistent_settings()
                .get_song_settings(&session_settings.current_song_name);
            settings.starred = true;
            println!(
                "{} is now starred. It will get chosen twice as often",
                session_settings.current_song_name
            );
            playlist_settings::update_song_settings(
                session_settings.current_song_name.clone(),
                settings,
            );
        }
        "unstar" => {
            let mut settings = playlist_settings::get_persistent_settings()
                .get_song_settings(&session_settings.current_song_name);
            if settings.starred {
                settings.starred = false;
                println!(
                    "{} is no longer starred",
                    session_settings.current_song_name
                );
            } else {
                println!("{} is not starred", session_settings.current_song_name);
            }
            playlist_settings::update_song_settings(
                session_settings.current_song_name.clone(),
                settings,
            );
        }
        "haslyrics" | "setlyrics" => {
            let mut settings = playlist_settings::get_persistent_settings()
                .get_song_settings(&session_settings.current_song_name);
            settings.has_lyrics = true;
            println!(
                "{} is set to have lyrics",
                session_settings.current_song_name
            );
            playlist_settings::update_song_settings(
                session_settings.current_song_name.clone(),
                settings,
            );
        }
        "hasnolyrics" | "setnolyrics" => {
            let mut settings = playlist_settings::get_persistent_settings()
                .get_song_settings(&session_settings.current_song_name);
            if settings.has_lyrics {
                settings.has_lyrics = false;
                println!(
                    "{} is no longer set to have lyrics",
                    session_settings.current_song_name
                );
            } else {
                println!(
                    "{} is not set to have lyrics",
                    session_settings.current_song_name
                );
            }
            playlist_settings::update_song_settings(
                session_settings.current_song_name.clone(),
                settings,
            );
        }
        "nolyrics" | "lyricsoff" | "nolyricsmode" | "deactivatelyrics" | "excludelyrics" => {
            session_settings.exclude_lyrics = true;
            println!("Songs with lyrics will now be excluded from the playlist");
        }
        "lyrics" | "lyricson" | "lyricsmode" | "activatelyrics" | "includelyrics" => {
            session_settings.exclude_lyrics = false;
            println!("songs with lyrics will now be included to the playlist");
        }
        "switchlyricsmode" | "l" => {
            switch_lyrics_mode(session_settings);
        }
        "playliststatus" => {
            println!("current song: {}", session_settings.current_song_name);
            println!(
                "playlist volume: {}",
                playlist_settings::get_persistent_settings().volume
            );
            if session_settings.is_muted {
                println!("playlist is muted");
            }
            if !session_settings.key_events_enabled {
                println!("keyboard shortcuts are disabled");
            }
            if !session_settings.shuffle {
                println!("playlist shuffling is disabled");
            }
            if session_settings.exclude_lyrics {
                println!("no lyrics mode is enabled");
            }
        }
        "songstatus" | "status" | "info" => {
            let song_settings = playlist_settings::get_persistent_settings()
                .get_song_settings(&session_settings.current_song_name);
            println!("current song: {}", session_settings.current_song_name);
            println!("song volume: {}", song_settings.song_volume);
            if song_settings.starred {
                println!("This song is starred");
            }
            println!("playlist index: {}", session_settings.current_song_index);
        }
        "index" | "playlistindex" => {
            println!("playlist index: {}", session_settings.current_song_index);
        }
        "songprobabilities" | "probabilities" | "showprobabilities" => {
            let settings = playlist_settings::get_persistent_settings();
            let mut probabilities = Vec::new();
            let mut sum = 0;
            for i in 0..paths.len() {
                let song_settings = settings.get_song_settings(&crate::get_song_name(&paths[i]));
                let p = session_settings.song_probability_distribution[i];
                let star_factor = if song_settings.starred { 2 } else { 1 };
                sum += p * star_factor;
                probabilities.push(p * star_factor);
            }
            let message = probabilities
                .into_iter()
                .enumerate()
                .map(|(i, p)| {
                    (
                        crate::get_song_name(&paths[i]),
                        p as f32 * 100.0 / sum as f32,
                    )
                })
                .map(|(song, p)| format!("{song} - {p:.2}%"))
                .collect::<Vec<String>>()
                .join("\n");
            println!("{message}");
        }
        "commands" | "help" => {
            let file_path = "commands.txt";
            let contents = fs::read_to_string(file_path).expect("Failed to read the file");
            println!("{contents}");
        }
        "terminate" | "exit" | "close" => {
            println!("closing audio player");
            process::exit(0);
        }
        msg if msg.starts_with("choosesong") => {
            let new_song = msg.split_once("choosesong").unwrap().1;
            match new_song.parse::<usize>() {
                Ok(index) => {
                    audio_player.clear();
                    let (source, song) = crate::index_song(paths, index);
                    audio_player.append(source);
                    audio_player.play();
                    println!("Now playing {song}");
                }
                Err(_) => match paths.iter().position(|song| crate::get_song_name(song).replace(' ', "") == new_song) {
                    Some(index) => {
                        audio_player.clear();
                        let (source, song) = crate::index_song(paths, index);
                        audio_player.append(source);
                        audio_player.play();
                        println!("Now playing {song}");
                    }
                    None => println!("this command requires a positive integer as an index or the name of a song in the playlist")
                }
            }
        }
        msg if msg.starts_with("choose") => {
            let new_song = msg.split_once("choose").unwrap().1;
            match new_song.parse::<usize>() {
                Ok(index) => {
                    audio_player.clear();
                    let (source, song) = crate::index_song(paths, index);
                    audio_player.append(source);
                    audio_player.play();
                    println!("Now playing {song}");
                }
                Err(_) => match paths.iter().position(|song| crate::get_song_name(song).replace(' ', "") == new_song) {
                    Some(index) => {
                        audio_player.clear();
                        let (source, song) = crate::index_song(paths, index);
                        audio_player.append(source);
                        audio_player.play();
                        println!("Now playing {song}");
                    }
                    None => println!("this command requires a positive integer as an index or the name of a song in the playlist")
                }
            }
        }
        msg if msg.starts_with('c') => {
            let new_song = msg.split_once('c').unwrap().1;
            match new_song.parse::<usize>() {
                Ok(index) => {
                    audio_player.clear();
                    let (source, song) = crate::index_song(paths, index);
                    audio_player.append(source);
                    audio_player.play();
                    println!("Now playing {song}");
                }
                Err(_) => match paths.iter().position(|song| crate::get_song_name(song).replace(' ', "") == new_song) {
                    Some(index) => {
                        audio_player.clear();
                        let (source, song) = crate::index_song(paths, index);
                        audio_player.append(source);
                        audio_player.play();
                        println!("Now playing {song}");
                    }
                    None => println!("this command requires a positive integer as an index or the name of a song in the playlist")
                }
            }
        }
        msg if msg.starts_with("play") => {
            let new_song = msg.split_once("play").unwrap().1;
            match new_song.parse::<usize>() {
                Ok(index) => {
                    audio_player.clear();
                    let (source, song) = crate::index_song(paths, index);
                    audio_player.append(source);
                    audio_player.play();
                    println!("Now playing {song}");
                }
                Err(_) => match paths.iter().position(|song| crate::get_song_name(song).replace(' ', "") == new_song) {
                    Some(index) => {
                        audio_player.clear();
                        let (source, song) = crate::index_song(paths, index);
                        audio_player.append(source);
                        audio_player.play();
                        println!("Now playing {song}");
                    }
                    None => println!("this command requires a positive integer as an index or the name of a song in the playlist")
                }
            }
        }
        msg if msg.starts_with("setvolume") || msg.starts_with("volume") => {
            let mut new_volume = msg.split_once("volume").unwrap().1;
            if new_volume.ends_with('%') {
                new_volume = new_volume.split_once('%').unwrap().0;
            }
            match new_volume.parse::<f32>() {
                Ok(new_volume) => {
                    let mut settings = playlist_settings::get_persistent_settings();
                    settings.volume = new_volume * 0.01;
                    let song_volume = playlist_settings::get_persistent_settings()
                        .get_song_settings(&session_settings.current_song_name)
                        .song_volume;
                    audio_player.set_volume(settings.volume * song_volume);
                    session_settings.is_muted = settings.volume == 0.0;
                    println!("playlist volume: {}%", (settings.volume * 100.0).round());
                    playlist_settings::update_settings(&settings);
                }
                Err(_) => println!("this command requires a number as volume like \"50%\""),
            }
        }
        msg if msg.starts_with('v') => {
            let mut new_volume = msg.split_once('v').unwrap().1.trim();
            if new_volume.ends_with('%') {
                new_volume = new_volume.split_once('%').unwrap().0;
            }
            match new_volume.parse::<f32>() {
                Ok(new_volume) => {
                    let mut settings = playlist_settings::get_persistent_settings();
                    settings.volume = new_volume * 0.01;
                    let song_volume = playlist_settings::get_persistent_settings()
                        .get_song_settings(&session_settings.current_song_name)
                        .song_volume;
                    audio_player.set_volume(settings.volume * song_volume);
                    println!("playlist volume: {}%", (settings.volume * 100.0).round());
                    playlist_settings::update_settings(&settings);
                }
                Err(_) => println!("this command requires a number as volume like \"50%\""),
            }
        }
        _ => println!("unknown command"),
    }
}

fn switch_lyrics_mode(session_settings: &mut SessionSettings) {
    if session_settings.exclude_lyrics {
        session_settings.exclude_lyrics = false;
        println!("Songs with lyrics will now be included to the playlist");
    } else {
        session_settings.exclude_lyrics = true;
        println!("Songs with lyrics will now be excluded from the playlist");
    }
}

fn restart_song(audio_player: &Sink, paths: &[PathBuf], session_settings: &mut SessionSettings) {
    audio_player.clear();
    let (source, song_name) = crate::index_song(paths, session_settings.current_song_index);
    audio_player.append(source);
    let song_settings = playlist_settings::get_persistent_settings().get_song_settings(&song_name);
    audio_player.set_volume(session_settings.playback_volume() * song_settings.song_volume);
    audio_player.play();
    println!("restarting {song_name}");
    session_settings.current_song_name = song_name;
}

fn next_song(audio_player: &Sink, paths: &[PathBuf], session_settings: &mut SessionSettings) {
    audio_player.clear();
    let (source, new_song, song_name) = crate::play_next_song(paths, session_settings);
    session_settings.current_song_index = new_song;
    audio_player.append(source);
    let song_settings = playlist_settings::get_persistent_settings().get_song_settings(&song_name);
    session_settings.current_song_name = song_name;
    audio_player.set_volume(session_settings.playback_volume() * song_settings.song_volume);
    audio_player.play();
}

fn decrease_volume(session_settings: &mut SessionSettings, audio_player: &Sink) {
    let mut settings = playlist_settings::get_persistent_settings();
    settings.volume -= 0.1;
    if settings.volume < 0.0 {
        settings.volume = 0.0;
    }
    let song_volume = playlist_settings::get_persistent_settings()
        .get_song_settings(&session_settings.current_song_name)
        .song_volume;
    audio_player.set_volume(settings.volume * song_volume);
    session_settings.is_muted = settings.volume == 0.0;
    println!("playlist volume: {}%", (settings.volume * 100.0).round());
    playlist_settings::update_settings(&settings);
}

fn increase_volume(session_settings: &mut SessionSettings, audio_player: &Sink) {
    let mut settings = playlist_settings::get_persistent_settings();
    settings.volume += 0.1;
    if settings.volume > 1.0 {
        settings.volume = 1.0;
    }
    let song_volume = playlist_settings::get_persistent_settings()
        .get_song_settings(&session_settings.current_song_name)
        .song_volume;
    audio_player.set_volume(settings.volume * song_volume);
    session_settings.is_muted = false;
    println!("playlist volume: {}%", (settings.volume * 100.0).round());
    playlist_settings::update_settings(&settings);
}

fn mute_or_unmute(session_settings: &mut SessionSettings, audio_player: &Sink) {
    if session_settings.is_muted {
        let song_volume = playlist_settings::get_persistent_settings()
            .get_song_settings(&session_settings.current_song_name)
            .song_volume;
        let volume = playlist_settings::get_persistent_settings().volume;
        audio_player.set_volume(volume * song_volume);
        session_settings.is_muted = false;
        println!("unmuted");
        println!("playlist volume: {}%", (volume * 100.0).round());
    } else {
        audio_player.set_volume(0.0);
        session_settings.is_muted = true;
        println!("muted");
    }
}

fn pause_or_play(audio_player: &Sink) {
    if audio_player.is_paused() {
        audio_player.play();
        println!("resumed");
    } else {
        audio_player.pause();
        println!("paused");
    }
}

pub fn handle_key_event(
    key_event: &Event,
    audio_player: &Sink,
    session_settings: &mut SessionSettings,
    paths: &[PathBuf],
) {
    let EventType::KeyPress(key) = key_event.event_type else {
        return;
    };
    match key {
        Key::F7 | Key::F4 => {
            pause_or_play(audio_player);
        }
        Key::F11 => {
            increase_volume(session_settings, audio_player);
        }
        Key::F10 => {
            decrease_volume(session_settings, audio_player);
        }
        Key::F12 => {
            mute_or_unmute(session_settings, audio_player);
        }
        Key::F8 => {
            next_song(audio_player, paths, session_settings);
        }
        Key::F6 => {
            restart_song(audio_player, paths, session_settings);
        }
        Key::F9 => {
            switch_lyrics_mode(session_settings);
        }
        _ => (),
    }
}
