use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};
use rodio::{Decoder, OutputStream, Sink};
use ratatui::widgets::ListState;
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::file::Track;

// Enum untuk melacak status pemutaran
#[derive(PartialEq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

pub struct App {
    pub running: bool,
    pub playlist: Vec<Track>,
    pub playlist_state: ListState,
    pub playback_state: PlaybackState,
    pub playback_position: Duration,
    pub shuffle_enabled: bool,
    last_tick: Option<Instant>,
    // Tambahkan field untuk audio
    _stream: OutputStream,
    sink: Sink,
}

impl App {
    pub fn new(playlist: Vec<Track>) -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let mut app = Self {
            running: true,
            playlist,
            playlist_state: ListState::default(),
            playback_state: PlaybackState::Stopped,
            playback_position: Duration::from_secs(0),
            shuffle_enabled: false,
            last_tick: None,
            _stream,
            sink,
        };

        if !app.playlist.is_empty() {
            app.playlist_state.select(Some(0));
        }

        app
    }

    pub fn update_playback(&mut self) {
        if let PlaybackState::Playing = self.playback_state {
            if let Some(last_tick) = self.last_tick {
                self.playback_position += last_tick.elapsed();
            }
            self.last_tick = Some(Instant::now());

            if self.sink.empty() {
                self.next_item();
                if self.playlist_state.selected() == Some(0) && !self.shuffle_enabled {
                    // Jika sudah di akhir playlist dan shuffle tidak aktif, berhenti
                    self.playback_state = PlaybackState::Stopped;
                    self.playback_position = Duration::from_secs(0);
                    self.last_tick = None;
                } else {
                    self.play_selected();
                }
            }
        }
    }

    pub fn next_item(&mut self) {
        if self.playlist.is_empty() { return; }
        if self.shuffle_enabled {
            let current_index = self.playlist_state.selected().unwrap_or(0);
            let mut other_indices: Vec<usize> = (0..self.playlist.len()).filter(|&i| i != current_index).collect();
            other_indices.shuffle(&mut thread_rng());
            self.playlist_state.select(other_indices.into_iter().next());
        } else {
            let i = self.playlist_state.selected().map_or(0, |i| {
                if i >= self.playlist.len() - 1 { 0 } else { i + 1 }
            });
            self.playlist_state.select(Some(i));
        }
        if !self.sink.is_paused() && self.playback_state != PlaybackState::Stopped {
            self.play_selected();
        }
    }

    pub fn previous_item(&mut self) {
        if self.playlist.is_empty() { return; }
        let i = self.playlist_state.selected().map_or(0, |i| {
            if i == 0 { self.playlist.len() - 1 } else { i - 1 }
        });
        self.playlist_state.select(Some(i));
        if !self.sink.is_paused() && self.playback_state != PlaybackState::Stopped {
            self.play_selected();
        }
    }

    pub fn play_selected(&mut self) {
        if let Some(selected_index) = self.playlist_state.selected() {
            if let Some(track) = self.playlist.get(selected_index) {
                self.sink.stop();
                self.sink.clear();

                if let Ok(file) = File::open(&track.path) {
                    let source = Decoder::new(BufReader::new(file)).unwrap();
                    self.sink.append(source);
                    self.sink.play();
                    self.playback_state = PlaybackState::Playing;
                    self.playback_position = Duration::from_secs(0);
                    self.last_tick = Some(Instant::now());
                }
            }
        }
    }

    pub fn toggle_playback(&mut self) {
        match self.playback_state {
            PlaybackState::Playing => {
                self.sink.pause();
                self.playback_state = PlaybackState::Paused;
                if let Some(last_tick) = self.last_tick.take() {
                    self.playback_position += last_tick.elapsed();
                }
            }
            PlaybackState::Paused => {
                self.sink.play();
                self.playback_state = PlaybackState::Playing;
                self.last_tick = Some(Instant::now());
            }
            PlaybackState::Stopped => {
                self.play_selected();
            }
        }
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle_enabled = !self.shuffle_enabled;
    }
}