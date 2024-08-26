use chrono::Local;
use rdev::Event;
use std::fmt::Display;

use crate::{
    playlist_settings::{AfterSong, SessionSettings},
    utils,
};

pub struct CrashReporter {
    session_settings: Option<SessionSettings>,
    crash_cause: CrashCause,
    enabled: bool,
}

pub enum CrashCause {
    Command(String),
    KeyEvent(Event),
    NextSong {
        next_song: String,
        after_song: AfterSong,
    },
    None,
}

impl CrashReporter {
    pub fn new() -> Self {
        Self {
            session_settings: None,
            crash_cause: CrashCause::None,
            enabled: true,
        }
    }

    pub fn set_session_settings(&mut self, session_settings: SessionSettings) {
        self.session_settings = Some(session_settings);
        self.crash_cause = CrashCause::None;
    }

    pub fn upcoming_command(&mut self, command: String, session_settings: SessionSettings) {
        self.session_settings = Some(session_settings);
        self.crash_cause = CrashCause::Command(command);
    }

    pub fn upcoming_key_event(&mut self, key_event: Event, session_settings: SessionSettings) {
        self.session_settings = Some(session_settings);
        self.crash_cause = CrashCause::KeyEvent(key_event);
    }

    pub fn next_song(&mut self, next_song: String, session_settings: SessionSettings) {
        let after_song = session_settings.after_song.clone();
        self.session_settings = Some(session_settings);
        self.crash_cause = CrashCause::NextSong {
            next_song,
            after_song,
        };
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

impl Display for CrashReporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "CRASH REPORT {}\n",
            Local::now().format("%H:%M %d.%m.%Y")
        )?;
        let Some(session_settings) = &self.session_settings else {
            return write!(f, "the program crashed during the program setup");
        };
        writeln!(
            f,
            "shuffling is {}\nexclude lyrics mode is {}\ncurrent song: {} ({}/{})\n",
            if session_settings.shuffle {
                "enabled"
            } else {
                "disabled"
            },
            if session_settings.exclude_lyrics {
                "enabled"
            } else {
                "disabled"
            },
            session_settings.current_song_name,
            utils::format_duration(&session_settings.song_progress()),
            session_settings.format_song_duration()
        )?;
        match &self.crash_cause {
            CrashCause::Command(command) => write!(f, "the error occurred as the command '{command}' was processed"),
            CrashCause::KeyEvent(key_event) => write!(f, "the error occurred as the key event '{key_event:?}' was processed"),
            CrashCause::NextSong {
                next_song,
                after_song,
            } => write!(f, "the error occurred while trying to play a new song\nsong: {next_song}\nafter song option: {after_song:?}"),
            CrashCause::None => write!(f, "the error did not occur during a command or after a song finished"),
        }
    }
}

impl Drop for CrashReporter {
    fn drop(&mut self) {
        if !self.enabled {
            return;
        }
        utils::write_to_file("crash-report.txt", &format!("{self}"));
        println!("program crashed! crash report can be found at crash-report.txt\nType anything to close the window");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read console input");
    }
}
