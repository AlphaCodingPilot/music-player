use std::{
    fs::{self, OpenOptions},
    io::Write,
};

use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};

pub struct SessionSettings {
    pub is_muted: bool,
    pub key_events_enabled: bool,
    pub shuffle: bool,
    pub exclude_lyrics: bool,
    pub current_song_index: usize,
    pub current_song_name: String,
    pub song_probability_distribution: Vec<u32>,
    pub random: ThreadRng,
}

impl SessionSettings {
    pub fn new(
        is_muted: bool,
        key_events_enabled: bool,
        shuffle: bool,
        exclude_lyrics: bool,
        current_song_index: usize,
        current_song_name: String,
        song_probability_distribution: Vec<u32>,
        random: ThreadRng,
    ) -> Self {
        Self {
            is_muted,
            key_events_enabled,
            shuffle,
            exclude_lyrics,
            current_song_index,
            current_song_name,
            song_probability_distribution,
            random,
        }
    }

    pub fn playback_volume(&self) -> f32 {
        if self.is_muted {
            0.0
        } else {
            get_persistent_settings().volume
        }
    }
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
    let json = to_json(settings);
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("playlist-settings.json")
        .expect("Failed to open file");
    file.write_all(json.as_bytes())
        .expect("Failed to write to file");
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
