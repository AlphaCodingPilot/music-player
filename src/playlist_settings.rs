use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};

use crate::{get_default_distribution, utils};

#[derive(Clone)]
pub struct SessionSettings {
    pub is_muted: bool,
    pub key_events_enabled: bool,
    pub shuffle: bool,
    pub exclude_lyrics: bool,
    pub current_song_index: usize,
    pub current_song_name: String,
    pub song_probability_distribution: Vec<u32>,
    pub song_start: Instant,
    song_progress: Duration,
    pub song_duration: Duration,
    pub after_song: AfterSong,
    pub random: ThreadRng,
}

impl SessionSettings {
    pub fn new(paths: &[PathBuf]) -> Self {
        Self {
            is_muted: false,
            key_events_enabled: true,
            shuffle: true,
            exclude_lyrics: false,
            current_song_index: 0,
            current_song_name: String::new(),
            song_probability_distribution: get_default_distribution(paths.len()),
            song_start: Instant::now(),
            song_progress: Duration::ZERO,
            song_duration: Duration::ZERO,
            after_song: AfterSong::Continue,
            random: rand::thread_rng(),
        }
    }

    pub fn playback_playlist_volume(&self) -> f32 {
        if self.is_muted {
            0.0
        } else {
            get_persistent_settings().volume
        }
    }

    pub fn song_progress(&self) -> Duration {
        self.song_progress + self.song_start.elapsed()
    }

    pub fn add_song_progress(&mut self, progress: Duration) {
        self.song_progress += progress;
    }
}

#[derive(Clone, Debug)]
pub enum AfterSong {
    Continue,
    Pause,
    PlaySong(usize),
}

#[derive(Serialize, Deserialize)]
pub struct PersistentSettings {
    pub volume: f32,
    song_settings: Vec<(String, SongSettings)>,
}

impl PersistentSettings {
    pub fn get_song_settings(&self, song: &str) -> SongSettings {
        self.song_settings
            .iter()
            .find(|(other, _)| song == other.as_str())
            .map(|(_, song_setting)| song_setting)
            .cloned()
            .unwrap_or_default()
    }
}

impl Default for PersistentSettings {
    fn default() -> Self {
        Self {
            volume: 1.0,
            song_settings: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SongSettings {
    pub song_volume: f32,
    pub starred: bool,
    pub has_lyrics: bool,
}

impl Default for SongSettings {
    fn default() -> Self {
        Self {
            song_volume: 0.5,
            starred: false,
            has_lyrics: false,
        }
    }
}

pub fn update_song_settings(song: String, settings: SongSettings) {
    let mut persistent_settings = get_persistent_settings();
    if let Some(index) = persistent_settings
        .song_settings
        .iter()
        .position(|(other, _)| &song == other)
    {
        persistent_settings.song_settings.remove(index);
    }
    persistent_settings.song_settings.push((song, settings));
    persistent_settings
        .song_settings
        .sort_by_key(|(song, _)| song.clone());
    update_settings(&persistent_settings);
}

pub fn update_settings(settings: &PersistentSettings) {
    utils::write_to_file("playlist-settings.json", &to_json(settings));
}

pub fn to_json(settings: &PersistentSettings) -> String {
    serde_json::to_string(settings).expect("json conversion failed")
}

pub fn get_persistent_settings() -> PersistentSettings {
    let file_path = "playlist-settings.json";
    let playlist_settings =
        fs::read_to_string(file_path).expect("Failed to read playlist-settings file");
    from_json(&playlist_settings)
}

fn from_json(json_str: &str) -> PersistentSettings {
    serde_json::from_str(json_str).expect("invalid json playlist-settings file")
}
