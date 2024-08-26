use std::{
    fs, path::PathBuf, time::{Duration, Instant}
};

use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};

use crate::utils;

#[derive(Clone)]
pub struct SessionSettings {
    pub is_muted: bool,
    pub key_events_enabled: bool,
    pub shuffle: bool,
    pub exclude_lyrics: bool,
    pub current_song_index: usize,
    pub current_song_name: String,
    pub duration_start: Instant,
    song_progress: Duration,
    pub song_duration: Option<Duration>,
    pub after_song: AfterSong,
    pub random: ThreadRng,
}

impl SessionSettings {
    pub fn playback_playlist_volume(&self) -> f32 {
        if self.is_muted {
            0.0
        } else {
            get_persistent_settings().volume
        }
    }

    pub fn song_progress(&self) -> Duration {
        self.song_progress + self.duration_start.elapsed()
    }

    pub fn format_song_duration(&self) -> String {
        match self.song_duration {
            Some(duration) => utils::format_duration(&duration),
            None => String::from("?"),
        }
    }

    pub fn add_song_progress(&mut self, progress: Duration) {
        self.song_progress += progress;
    }
    
    pub fn reset_song_progress(&mut self) {
        self.song_progress = Duration::ZERO;
    }
}

impl Default for SessionSettings {
    fn default() -> Self {
        Self {
            is_muted: false,
            key_events_enabled: true,
            shuffle: true,
            exclude_lyrics: false,
            current_song_index: 0,
            current_song_name: String::new(),
            duration_start: Instant::now(),
            song_progress: Duration::ZERO,
            song_duration: None,
            after_song: AfterSong::Continue,
            random: rand::thread_rng(),
        }
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
    song_probability_distribution: Vec<(String, u32)>,
    song_play_count: Vec<(String, u32)>,
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

    pub fn get_probability_distribution(&self, paths: &[PathBuf]) -> Vec<u32> {
        let mut probabilities = crate::get_default_distribution(paths.len());
        for (song, probability) in &self.song_probability_distribution {
            if let Some(index) = paths.iter().position(|path| path.to_str().expect("path does not have a name") == song.as_str()) {
                probabilities[index] = *probability;
            }
        }
        probabilities
    }

    pub fn set_song_probability(&mut self, song: &str, probability: u32) {
        if let Some(index) = self.song_probability_distribution.iter().position(|(other, _)| song == other) {
            self.song_probability_distribution[index].1 = probability;
            return;
        }
        self.song_probability_distribution.push((song.to_string(), probability));
    }

    pub fn get_song_play_count(&self, song: &str) -> u32 {
        let index = self.song_play_count.iter().position(|(other, _)| song == other);
        match index {
            Some(index) => self.song_play_count[index].1,
            None => 0,
        }
    }

    pub fn accumulate_play_count(&mut self, song: &str) {
        let index = self.song_play_count.iter().position(|(other, _)| song == other);
        match index {
            Some(index) => self.song_play_count[index].1 += 1,
            None => self.song_play_count.push((song.to_string(), 1)),
        }
    }
}

impl Default for PersistentSettings {
    fn default() -> Self {
        Self {
            volume: 1.0,
            song_settings: Vec::new(),
            song_probability_distribution: Vec::new(),
            song_play_count: Vec::new(),
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
