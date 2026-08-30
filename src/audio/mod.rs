pub mod backend;
pub mod duration;
pub mod probe;
pub mod sync;
pub mod types;

pub use backend::{detect_backend, AudioBackend};
pub use duration::parse_duration_to_secs;
pub use probe::probe_exact_duration;
pub use sync::{spawn_playhead_monitor, PlayheadUpdate};
pub use types::PlaybackState;

use std::process::{Child, Command, Stdio};
use std::sync::mpsc::Receiver;
use std::time::Instant;

pub struct AudioPlayer {
    pub backend: Option<AudioBackend>,
    child: Option<Child>,
    pub state: PlaybackState,
    pub current_title: Option<String>,
    pub current_url: Option<String>,
    pub elapsed_secs: u64,
    pub total_duration_secs: Option<u64>,
    pub last_tick: Option<Instant>,
    pub probe_rx: Option<Receiver<u64>>,
    pub playhead_rx: Option<Receiver<PlayheadUpdate>>,
    pub is_buffering: bool,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioPlayer {
    pub fn new() -> Self {
        Self {
            backend: detect_backend(),
            child: None,
            state: PlaybackState::Stopped,
            current_title: None,
            current_url: None,
            elapsed_secs: 0,
            total_duration_secs: None,
            last_tick: None,
            probe_rx: None,
            playhead_rx: None,
            is_buffering: false,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.state == PlaybackState::Playing
    }

    pub fn is_active(&self) -> bool {
        self.state != PlaybackState::Stopped
    }

    pub fn play(&mut self, title: &str, url: &str, duration_str: Option<&str>) -> bool {
        let total_duration_secs = duration_str.and_then(parse_duration_to_secs);
        self.play_at_offset(title, url, 0, total_duration_secs)
    }

    pub fn play_at_offset(
        &mut self,
        title: &str,
        url: &str,
        start_secs: u64,
        total_duration_secs: Option<u64>,
    ) -> bool {
        self.stop();

        let backend = match self.backend {
            Some(b) => b,
            None => return false,
        };

        match backend::spawn_player(backend, url, start_secs) {
            Ok(mut child) => {
                let playhead_rx = child.stderr.take().map(spawn_playhead_monitor);
                self.child = Some(child);
                self.state = PlaybackState::Playing;
                self.current_title = Some(title.to_string());
                self.current_url = Some(url.to_string());
                self.elapsed_secs = start_secs;
                self.total_duration_secs = total_duration_secs;
                self.last_tick = Some(Instant::now());
                self.is_buffering = playhead_rx.is_some();
                self.playhead_rx = playhead_rx;

                let (tx, rx) = std::sync::mpsc::channel();
                let probe_url = url.to_string();
                std::thread::spawn(move || {
                    if let Some(secs) = probe_exact_duration(&probe_url) {
                        let _ = tx.send(secs);
                    }
                });
                self.probe_rx = Some(rx);

                true
            }
            Err(_) => {
                self.state = PlaybackState::Stopped;
                false
            }
        }
    }

    pub fn seek(&mut self, delta_secs: i64) -> bool {
        if !self.is_active() {
            return false;
        }

        let new_offset = if delta_secs >= 0 {
            self.elapsed_secs.saturating_add(delta_secs as u64)
        } else {
            self.elapsed_secs.saturating_sub((-delta_secs) as u64)
        };

        let target_offset = if let Some(total) = self.total_duration_secs {
            new_offset.min(total)
        } else {
            new_offset
        };

        if let Some(child) = &mut self.child {
            if self.backend == Some(AudioBackend::Mpv) {
                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    let cmd = format!("seek {}\n", delta_secs);
                    if stdin.write_all(cmd.as_bytes()).is_ok() && stdin.flush().is_ok() {
                        self.elapsed_secs = target_offset;
                        self.last_tick = Some(Instant::now());
                        return true;
                    }
                }
            }
        }

        if let (Some(title), Some(url)) = (self.current_title.clone(), self.current_url.clone()) {
            let total = self.total_duration_secs;
            let was_paused = self.state == PlaybackState::Paused;
            let success = self.play_at_offset(&title, &url, target_offset, total);
            if success && was_paused {
                self.pause();
            }
            return success;
        }

        false
    }

    pub fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            if let Some(child) = &self.child {
                let _ = Command::new("kill")
                    .args(["-STOP", &child.id().to_string()])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .output();
                self.state = PlaybackState::Paused;
                self.last_tick = None;
            }
        }
    }

    pub fn resume(&mut self) {
        if self.state == PlaybackState::Paused {
            if let Some(child) = &self.child {
                let _ = Command::new("kill")
                    .args(["-CONT", &child.id().to_string()])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .output();
                self.state = PlaybackState::Playing;
                self.last_tick = Some(Instant::now());
            }
        }
    }

    pub fn toggle_pause(&mut self) {
        match self.state {
            PlaybackState::Playing => self.pause(),
            PlaybackState::Paused => self.resume(),
            PlaybackState::Stopped => {}
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.state = PlaybackState::Stopped;
        self.current_title = None;
        self.current_url = None;
        self.elapsed_secs = 0;
        self.total_duration_secs = None;
        self.last_tick = None;
        self.probe_rx = None;
        self.playhead_rx = None;
        self.is_buffering = false;
    }

    pub fn poll_status(&mut self) {
        if let Some(rx) = &self.probe_rx {
            if let Ok(exact_secs) = rx.try_recv() {
                self.total_duration_secs = Some(exact_secs);
                self.probe_rx = None;
            }
        }

        if let Some(rx) = &self.playhead_rx {
            while let Ok(update) = rx.try_recv() {
                self.is_buffering = update.is_buffering;
                if let Some(sec) = update.exact_playhead {
                    self.elapsed_secs = sec.round() as u64;
                    self.last_tick = Some(Instant::now());
                }
            }
        }

        if self.state == PlaybackState::Playing && !self.is_buffering {
            if let Some(last) = self.last_tick {
                let now = Instant::now();
                let delta = now.duration_since(last).as_secs();
                if delta > 0 {
                    self.elapsed_secs = self.elapsed_secs.saturating_add(delta);
                    self.last_tick = Some(now);
                }
            } else {
                self.last_tick = Some(Instant::now());
            }
        }

        if let Some(child) = &mut self.child {
            if let Ok(Some(_)) = child.try_wait() {
                self.child = None;
                self.state = PlaybackState::Stopped;
                self.current_title = None;
                self.current_url = None;
                self.elapsed_secs = 0;
                self.total_duration_secs = None;
                self.last_tick = None;
                self.probe_rx = None;
                self.playhead_rx = None;
                self.is_buffering = false;
            }
        }
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        self.stop();
    }
}
