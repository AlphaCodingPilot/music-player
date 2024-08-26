use std::{fs, path::PathBuf, process, time::Instant};

use rdev::{Event, EventType, Key};
use rodio::{Sink, Source};

use crate::{
    crash_reporter::CrashReporter,
    playlist_settings::{self, AfterSong, SessionSettings},
    utils,
};

pub fn handle_console_commands(
    input_buffer: &str,
    audio_player: &Sink,
    session_settings: &mut SessionSettings,
    paths: &[PathBuf],
    crash_reporter: &mut CrashReporter,
) {
    let input = input_buffer.trim().to_lowercase();
    let input = input.replace(' ', "");
    match input.as_str() {
        "" => (),
        "pause" => {
            pause(session_settings, audio_player);
        }
        "resume" | "r" | "play" => {
            resume(session_settings, audio_player);
        }
        "p" | "k" => {
            pause_or_play(session_settings, audio_player);
        }
        "mute" => {
            mute(session_settings, audio_player);
        }
        "unmute" => {
            unmute(session_settings, audio_player);
        }
        "m" => {
            switch_muted(session_settings, audio_player);
        }
        "volume+" | "v+" | "+" => {
            increase_volume(session_settings, audio_player);
        }
        "volume- " | "v-" | "-" => {
            decrease_volume(session_settings, audio_player);
        }
        "songvolume+" | "sv+" => {
            increase_song_volume(session_settings, audio_player);
        }
        "songvolume-" | "sv-" => {
            decrease_song_volume(session_settings, audio_player);
        }
        "volume" | "v" => {
            print_volume(session_settings);
        }
        "nextsong" | "next" | "n" | "skip" => {
            next_song(audio_player, paths, session_settings, crash_reporter);
        }
        "restartsong" | "restart" | "rs" => {
            restart_song(audio_player, paths, session_settings);
        }
        "start" => {
            go_to_first_song(audio_player, paths, session_settings);
        }
        "pauseaftersong" | "pauseaftercurrentsong" | "pausenext" => {
            pause_after_song(session_settings);
        }
        "continueaftersong" | "resetaftersong" => {
            continue_after_song(session_settings);
        }
        "playlist" => {
            print_playlist(paths);
        }
        "enablekeyboard" | "ekb" => {
            enable_keyboard_input(session_settings);
        }
        "disablekeyboard" | "dkb" => {
            disable_keyboard_input(session_settings);
        }
        "keyboard" | "kb" | "ks" => {
            switch_keyboard_input(session_settings);
        }
        "enableshuffle" | "es" => {
            enable_shuffling(session_settings, paths);
        }
        "disableshuffle" | "ds" => {
            disable_shuffling(session_settings);
        }
        "shuffle" | "sh" => {
            switch_shuffling(session_settings, paths);
        }
        "resetprobabilities" => {
            reset_probabilities(paths);
        }
        "star" | "s" => {
            star(session_settings);
        }
        "unstar" => {
            unstar(session_settings);
        }
        "starred" | "starredsongs" => {
            print_starred_songs(paths);
        }
        "haslyrics" | "setlyrics" => {
            set_lyrics(session_settings);
        }
        "hasnolyrics" | "setnolyrics" => {
            set_no_lyrics(session_settings);
        }
        "nolyrics" | "lyricsoff" | "nolyricsmode" | "deactivatelyrics" | "excludelyrics"
        | "focusmode" | "focus" | "focusmodeon" => {
            turn_off_lyrics_mode(session_settings);
        }
        "lyrics" | "lyricson" | "lyricsmode" | "activatelyrics" | "includelyrics"
        | "focusmodeoff" => {
            turn_on_lyrics_mode(session_settings);
        }
        "switchlyricsmode" | "l" => {
            switch_lyrics_mode(session_settings);
        }
        "status" | "info" => {
            print_status(session_settings, paths);
        }
        "index" | "playlistindex" => {
            print_index(session_settings);
        }
        "progress" | "songprogress" => {
            print_progress(session_settings);
        }
        "songprobabilities" | "probabilities" | "showprobabilities" => {
            print_song_probabilities(paths);
        }
        "playcount" => {
            print_play_count(&session_settings.current_song_name);
        }
        "commands" | "help" => {
            print_commands();
        }
        "terminate" | "exit" | "close" => {
            exit_program(crash_reporter);
        }
        msg if msg.starts_with("choosesong") => {
            let new_song = msg.split_once("choosesong").unwrap().1;
            choose_song(new_song, audio_player, paths, session_settings);
        }
        msg if msg.starts_with("choose") => {
            let new_song = msg.split_once("choose").unwrap().1;
            choose_song(new_song, audio_player, paths, session_settings);
        }
        msg if msg.starts_with('c') => {
            let new_song = msg.split_once('c').unwrap().1;
            choose_song(new_song, audio_player, paths, session_settings);
        }
        msg if msg.starts_with("play") => {
            let new_song = msg.split_once("play").unwrap().1;
            choose_song(new_song, audio_player, paths, session_settings);
        }
        msg if msg.starts_with("setvolume") || msg.starts_with("volume") => {
            let volume = msg.split_once("volume").unwrap().1;
            set_volume(volume, session_settings, audio_player);
        }
        msg if msg.starts_with('v') => {
            let volume = msg.split_once('v').unwrap().1.trim();
            set_volume(volume, session_settings, audio_player);
        }
        msg if msg.starts_with("songvolume") => {
            let volume = msg.split_once("songvolume").unwrap().1;
            set_song_volume(volume, session_settings, audio_player);
        }
        msg if msg.starts_with("sv") => {
            let volume = msg.split_once("sv").unwrap().1;
            set_song_volume(volume, session_settings, audio_player);
        }
        msg if msg.starts_with("nextsong") => {
            let next_song = msg.split_once("nextsong").unwrap().1;
            choose_next_song(session_settings, next_song, paths);
        }
        msg if msg.starts_with("choosenextsong") => {
            let next_song = msg.split_once("choosenextsong").unwrap().1;
            choose_next_song(session_settings, next_song, paths);
        }
        msg if msg.starts_with("aftersong") => {
            let next_song = msg.split_once("aftersong").unwrap().1;
            choose_next_song(session_settings, next_song, paths);
        }
        msg if msg.starts_with("next") => {
            let next_song = msg.split_once("next").unwrap().1;
            choose_next_song(session_settings, next_song, paths);
        }
        _ => println!("unknown command"),
    }
}

pub fn handle_key_event(
    key_event: &Event,
    audio_player: &Sink,
    session_settings: &mut SessionSettings,
    paths: &[PathBuf],
    crash_reporter: &mut CrashReporter,
) {
    let EventType::KeyPress(key) = key_event.event_type else {
        return;
    };
    if !session_settings.key_events_enabled {
        return;
    }
    match key {
        Key::F7 | Key::F4 => {
            pause_or_play(session_settings, audio_player);
        }
        Key::F11 => {
            increase_volume(session_settings, audio_player);
        }
        Key::F10 => {
            decrease_volume(session_settings, audio_player);
        }
        Key::F12 => {
            switch_muted(session_settings, audio_player);
        }
        Key::F8 => {
            next_song(audio_player, paths, session_settings, crash_reporter);
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

fn pause(session_settings: &mut SessionSettings, audio_player: &Sink) {
    if !audio_player.is_paused() {
        audio_player.pause();
        session_settings.add_song_progress(session_settings.duration_start.elapsed());
        println!("paused");
    }
}

fn resume(session_settings: &mut SessionSettings, audio_player: &Sink) {
    if audio_player.is_paused() {
        audio_player.play();
        session_settings.duration_start = Instant::now();
        println!("resumed");
    }
}

fn mute(session_settings: &mut SessionSettings, audio_player: &Sink) {
    if !session_settings.is_muted {
        audio_player.set_volume(0.0);
        session_settings.is_muted = true;
        println!("muted");
    }
}

fn unmute(session_settings: &mut SessionSettings, audio_player: &Sink) {
    if session_settings.is_muted {
        let song_volume = playlist_settings::get_persistent_settings()
            .get_song_settings(&session_settings.current_song_name)
            .song_volume;
        let volume = playlist_settings::get_persistent_settings().volume;
        audio_player.set_volume(volume * song_volume);
        session_settings.is_muted = false;
        println!("unmuted");
        println!("playlist volume: {}%", volume * 100.0);
    }
}

fn increase_song_volume(session_settings: &SessionSettings, audio_player: &Sink) {
    let mut settings = playlist_settings::get_persistent_settings()
        .get_song_settings(&session_settings.current_song_name);
    settings.song_volume += 0.1;
    if settings.song_volume > 1.0 {
        settings.song_volume = 1.0;
    }
    let playlist_volume = playlist_settings::get_persistent_settings().volume;
    if !session_settings.is_muted {
        audio_player.set_volume(settings.song_volume * playlist_volume);
    }
    println!("song volume: {}%", settings.song_volume * 100.0);
    playlist_settings::update_song_settings(session_settings.current_song_name.clone(), settings);
}

fn decrease_song_volume(session_settings: &SessionSettings, audio_player: &Sink) {
    let mut settings = playlist_settings::get_persistent_settings()
        .get_song_settings(&session_settings.current_song_name);
    settings.song_volume -= 0.1;
    if settings.song_volume < 0.0 {
        settings.song_volume = 0.0;
    }
    let playlist_volume = playlist_settings::get_persistent_settings().volume;
    audio_player.set_volume(settings.song_volume * playlist_volume);
    println!("song volume: {}%", settings.song_volume * 100.0);
    playlist_settings::update_song_settings(session_settings.current_song_name.clone(), settings);
}

fn print_volume(session_settings: &SessionSettings) {
    let volume = playlist_settings::get_persistent_settings().volume;
    let song_volume = playlist_settings::get_persistent_settings()
        .get_song_settings(&session_settings.current_song_name)
        .song_volume;
    println!("playlist volume: {}%", volume * 100.0);
    println!("song volume: {}%", song_volume * 100.0);
    println!("combined volume: {}%", volume * song_volume * 100.0);
    if session_settings.is_muted {
        println!("The playlist is muted!");
    }
}

fn go_to_first_song(
    audio_player: &Sink,
    paths: &[PathBuf],
    session_settings: &mut SessionSettings,
) {
    choose_song_by_index(audio_player, paths, 0, session_settings);
}

fn print_playlist(paths: &[PathBuf]) {
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

fn enable_keyboard_input(session_settings: &mut SessionSettings) {
    if !session_settings.key_events_enabled {
        session_settings.key_events_enabled = true;
        println!("keyboard shortcuts enabled");
    }
}

fn disable_keyboard_input(session_settings: &mut SessionSettings) {
    if !session_settings.key_events_enabled {
        session_settings.key_events_enabled = false;
        println!("keyboard shortcuts disabled");
    }
}

fn switch_keyboard_input(session_settings: &mut SessionSettings) {
    if session_settings.key_events_enabled {
        session_settings.key_events_enabled = false;
        println!("keyboard shortcuts disabled");
    } else {
        session_settings.key_events_enabled = true;
        println!("keyboard shortcuts enabled");
    }
}

fn enable_shuffling(session_settings: &mut SessionSettings, paths: &[PathBuf]) {
    let mut persistent_settings = playlist_settings::get_persistent_settings();
    if !session_settings.shuffle {
        session_settings.shuffle = true;
        for i in 0..paths.len() {
            persistent_settings.set_song_probability(paths[i].to_str().expect("path has no name"), 1);
        }
        println!("shuffle playlist enabled");
    }
    playlist_settings::update_settings(&persistent_settings);
}

fn disable_shuffling(session_settings: &mut SessionSettings) {
    if !session_settings.shuffle {
        session_settings.shuffle = false;
        println!("shuffle playlist disabled");
    }
}

fn switch_shuffling(session_settings: &mut SessionSettings, paths: &[PathBuf]) {
    let mut persistent_settings = playlist_settings::get_persistent_settings();
    if session_settings.shuffle {
        session_settings.shuffle = false;
        for i in 0..paths.len() {
            persistent_settings.set_song_probability(paths[i].to_str().expect("path has no name"), 1);
        }
        println!("shuffle playlist disabled");
    } else {
        session_settings.shuffle = true;
        println!("shuffle playlist enabled");
    }
    playlist_settings::update_settings(&persistent_settings);
}

fn reset_probabilities(paths: &[PathBuf]) {
    let mut persistent_settings = playlist_settings::get_persistent_settings();
    for i in 0..paths.len() {
        persistent_settings.set_song_probability(paths[i].to_str().expect("path has no name"), 1);
    }
    playlist_settings::update_settings(&persistent_settings);
}

fn star(session_settings: &SessionSettings) {
    let mut settings = playlist_settings::get_persistent_settings()
        .get_song_settings(&session_settings.current_song_name);
    settings.starred = true;
    println!(
        "{} is now starred. It will get chosen twice as often",
        session_settings.current_song_name
    );
    playlist_settings::update_song_settings(session_settings.current_song_name.clone(), settings);
}

fn unstar(session_settings: &SessionSettings) {
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
    playlist_settings::update_song_settings(session_settings.current_song_name.clone(), settings);
}

fn print_starred_songs(paths: &[PathBuf]) {
    println!("starred songs:");
    let settings = playlist_settings::get_persistent_settings();
    for path in paths {
        let song = crate::get_song_name(path);
        if settings.get_song_settings(&song).starred {
            println!("{song}");
        }
    }
}

fn set_lyrics(session_settings: &SessionSettings) {
    let mut settings = playlist_settings::get_persistent_settings()
        .get_song_settings(&session_settings.current_song_name);
    settings.has_lyrics = true;
    println!(
        "{} is set to have lyrics",
        session_settings.current_song_name
    );
    playlist_settings::update_song_settings(session_settings.current_song_name.clone(), settings);
}

fn set_no_lyrics(session_settings: &SessionSettings) {
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
    playlist_settings::update_song_settings(session_settings.current_song_name.clone(), settings);
}

fn turn_off_lyrics_mode(session_settings: &mut SessionSettings) {
    session_settings.exclude_lyrics = true;
    println!("Songs with lyrics will now be excluded from the playlist");
}

fn turn_on_lyrics_mode(session_settings: &mut SessionSettings) {
    session_settings.exclude_lyrics = false;
    println!("songs with lyrics will now be included to the playlist");
}

fn print_status(session_settings: &SessionSettings, paths: &[PathBuf]) {
    println!("current song: {}", session_settings.current_song_name);
    let persistent_settings = playlist_settings::get_persistent_settings();
    println!("playlist volume: {}", persistent_settings.volume);
    if session_settings.is_muted {
        println!("playlist is muted");
    }
    match session_settings.after_song {
        AfterSong::Pause => println!("the playlist will pause after the current song"),
        AfterSong::PlaySong(next_song) => println!(
            "the next song is set as {}",
            crate::get_song_name(&paths[next_song])
        ),
        AfterSong::Continue => (),
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
    let song_settings = persistent_settings.get_song_settings(&session_settings.current_song_name);
    println!(
        "progress: ({})",
        session_settings.format_song_duration()
    );
    print_play_count(&session_settings.current_song_name);
    println!(
        "song volume: {}% (playing at {}%)",
        song_settings.song_volume,
        session_settings.playback_playlist_volume()
    );
    if song_settings.starred {
        println!("this song is starred");
    }
    if song_settings.has_lyrics {
        println!("this song has lyrics");
    }
    println!("playlist index: {}", session_settings.current_song_index);
}

fn print_index(session_settings: &SessionSettings) {
    println!("playlist index: {}", session_settings.current_song_index);
}

fn print_progress(session_settings: &SessionSettings) {
    match &session_settings.song_duration {
        Some(duration) => println!(
            "{}: {}/{} ({}%)",
            session_settings.current_song_name,
            utils::format_duration(&session_settings.song_progress()),
            utils::format_duration(duration),
            ((session_settings.song_progress().as_secs_f32()
            / duration.as_secs_f32())
            * 100.0)
            .round()
        ),
        None => println!("{}: {}", session_settings.current_song_name, utils::format_duration(&session_settings.song_progress())),
    }
}

fn print_song_probabilities(paths: &[PathBuf]) {
    let settings = playlist_settings::get_persistent_settings();
    let mut probabilities = Vec::new();
    let mut sum = 0;
    for i in 0..paths.len() {
        let song_settings = settings.get_song_settings(&crate::get_song_name(&paths[i]));
        let base_probability = settings.get_probability_distribution(paths)[i];
        let p = base_probability;
        let star_factor = if song_settings.starred { 2 } else { 1 };
        sum += p * star_factor;
        probabilities.push((p * star_factor, base_probability));
    }
    let message = probabilities
        .into_iter()
        .enumerate()
        .map(|(i, p)| {
            (
                crate::get_song_name(&paths[i]),
                p.0 as f32 * 100.0 / sum as f32,
                p.1,
            )
        })
        .map(|(song, percentage, base)| format!("{song} - {percentage:.2}% ({base})"))
        .collect::<Vec<String>>()
        .join("\n");
    println!("{message}");
}

fn print_commands() {
    let file_path = "commands.txt";
    let contents = fs::read_to_string(file_path).expect("Failed to read the file");
    println!("{contents}");
}

fn exit_program(crash_reporter: &mut CrashReporter) {
    println!("closing audio player");
    crash_reporter.disable();
    process::exit(0);
}

fn set_song_volume(volume: &str, session_settings: &SessionSettings, audio_player: &Sink) {
    let mut new_volume = volume;
    if new_volume.ends_with('%') {
        new_volume = new_volume.split_once('%').unwrap().0;
    }
    match new_volume.parse::<f32>() {
        Ok(new_volume) => {
            let mut settings = playlist_settings::get_persistent_settings()
                .get_song_settings(&session_settings.current_song_name);
            settings.song_volume = new_volume * 0.01;
            let playlist_volume = playlist_settings::get_persistent_settings().volume;
            if !session_settings.is_muted {
                audio_player.set_volume(settings.song_volume * playlist_volume);
            }
            println!("song volume: {}%", settings.song_volume * 100.0);
            playlist_settings::update_song_settings(
                session_settings.current_song_name.clone(),
                settings,
            );
        }
        Err(_) => println!("this command requires a number as volume like \"50%\""),
    }
}

fn set_volume(volume: &str, session_settings: &mut SessionSettings, audio_player: &Sink) {
    let mut new_volume = volume;
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
            println!("playlist volume: {}%", settings.volume * 100.0);
            playlist_settings::update_settings(&settings);
        }
        Err(_) => println!("this command requires a number as volume like \"50%\""),
    }
}

fn choose_song(
    new_song: &str,
    audio_player: &Sink,
    paths: &[PathBuf],
    session_settings: &mut SessionSettings,
) {
    match new_song.parse::<usize>() {
        Ok(index) => {
            if index >= paths.len() {
                println!("the given index does not exist in the playlist");
                return;
            }
            choose_song_by_index(audio_player, paths, index, session_settings);
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

fn choose_song_by_index(
    audio_player: &Sink,
    paths: &[PathBuf],
    index: usize,
    session_settings: &mut SessionSettings,
) {
    audio_player.clear();
    let (source, file_name) = crate::index_song(paths, index);
    session_settings.song_duration = source
        .total_duration();
    println!(
        "Now playing: {file_name} ({})",
        session_settings.format_song_duration(),
    );
    let mut settings = playlist_settings::get_persistent_settings();
    let song_settings = settings.get_song_settings(&file_name);
    let song_volume = song_settings.song_volume;
    let playlist_volume = settings.volume;
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
    audio_player.append(source);
    let song_settings = settings.get_song_settings(&file_name);
    let track_volume = song_settings.song_volume * settings.volume;
    audio_player.set_volume(track_volume);
    audio_player.play();
    session_settings.current_song_index = index;
    session_settings.current_song_name = file_name;
    session_settings.duration_start = Instant::now();
    session_settings.after_song = AfterSong::Continue;
    if session_settings.shuffle {
        let mut choosable_songs = 0;
        for i in 0..paths.len() {
            settings.set_song_probability(paths[i].to_str().expect("path has no name"), settings.get_probability_distribution(paths)[i] + 1);
            if !session_settings.exclude_lyrics
                || !settings
                    .get_song_settings(&crate::get_song_name(&paths[i]))
                    .has_lyrics
            {
                choosable_songs += 1;
            }
        }
        settings.set_song_probability(paths[index].to_str().expect("path has no name"), 0);
        if choosable_songs == 1 {
            //if the song that was last played is the only song that can be played, it can be chosen again
            settings.set_song_probability(paths[index].to_str().expect("path has no name"), 1);
        }
    }
    playlist_settings::update_settings(&settings);
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
    audio_player
        .set_volume(session_settings.playback_playlist_volume() * song_settings.song_volume);
    audio_player.play();
    println!("restarting {song_name}");
    session_settings.current_song_name = song_name;
}

fn next_song(
    audio_player: &Sink,
    paths: &[PathBuf],
    session_settings: &mut SessionSettings,
    crash_reporter: &mut CrashReporter,
) {
    audio_player.clear();
    let index = crate::get_next_song_index(session_settings, paths);
    crash_reporter.next_song(
        crate::get_song_name(&paths[index]),
        session_settings.clone(),
    );
    let (source, _, song_name) = crate::play_next_song(index, paths, session_settings);
    audio_player.append(source);
    let song_settings = playlist_settings::get_persistent_settings().get_song_settings(&song_name);
    audio_player
        .set_volume(session_settings.playback_playlist_volume() * song_settings.song_volume);
    session_settings.after_song = AfterSong::Continue;
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
    println!("playlist volume: {}%", settings.volume * 100.0);
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
    println!("playlist volume: {}%", settings.volume * 100.0);
    playlist_settings::update_settings(&settings);
}

fn switch_muted(session_settings: &mut SessionSettings, audio_player: &Sink) {
    if session_settings.is_muted {
        let song_volume = playlist_settings::get_persistent_settings()
            .get_song_settings(&session_settings.current_song_name)
            .song_volume;
        let volume = playlist_settings::get_persistent_settings().volume;
        audio_player.set_volume(volume * song_volume);
        session_settings.is_muted = false;
        println!("unmuted");
        println!("playlist volume: {}%", volume * 100.0);
    } else {
        audio_player.set_volume(0.0);
        session_settings.is_muted = true;
        println!("muted");
    }
}

fn pause_or_play(session_settings: &mut SessionSettings, audio_player: &Sink) {
    if audio_player.is_paused() {
        resume(session_settings, audio_player);
    } else {
        pause(session_settings, audio_player);
    }
}

fn pause_after_song(session_settings: &mut SessionSettings) {
    session_settings.after_song = AfterSong::Pause;
}

fn choose_next_song(session_settings: &mut SessionSettings, next_song: &str, paths: &[PathBuf]) {
    match next_song.parse::<usize>() {
        Ok(index) => {
            if index >= paths.len() {
                println!("the given index does not exist in the playlist");
                return;
            }
            session_settings.after_song = AfterSong::PlaySong(index);
        }
        Err(_) => match paths.iter().position(|song| crate::get_song_name(song).replace(' ', "") == next_song) {
            Some(index) => {
                session_settings.after_song = AfterSong::PlaySong(index);
            }
            None => println!("this command requires a positive integer as an index or the name of a song in the playlist")
        }
    }
}

fn continue_after_song(session_settings: &mut SessionSettings) {
    session_settings.after_song = AfterSong::Continue;
}

fn print_play_count(song: &str) {
    let play_count = playlist_settings::get_persistent_settings().get_song_play_count(song);
    println!("{song} has been played {play_count} times");
}
