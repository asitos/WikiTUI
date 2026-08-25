use std::process::{Child, Command, Stdio};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioBackend {
    Mpv,
    Ffplay,
    Cvlc,
    Vlc,
    Afplay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

pub struct AudioPlayer {
    pub backend: Option<AudioBackend>,
    child: Option<Child>,
    pub state: PlaybackState,
    pub current_title: Option<String>,
    pub current_url: Option<String>,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioPlayer {
    pub fn new() -> Self {
        Self {
            backend: Self::detect_backend(),
            child: None,
            state: PlaybackState::Stopped,
            current_title: None,
            current_url: None,
        }
    }

    pub fn detect_backend() -> Option<AudioBackend> {
        if Self::has_binary("mpv") {
            Some(AudioBackend::Mpv)
        } else if Self::has_binary("ffplay") {
            Some(AudioBackend::Ffplay)
        } else if Self::has_binary("cvlc") {
            Some(AudioBackend::Cvlc)
        } else if Self::has_binary("vlc") {
            Some(AudioBackend::Vlc)
        } else if Self::has_binary("afplay") {
            Some(AudioBackend::Afplay)
        } else {
            None
        }
    }

    fn has_binary(bin: &str) -> bool {
        Command::new("which")
            .arg(bin)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    pub fn is_playing(&self) -> bool {
        self.state == PlaybackState::Playing
    }

    pub fn is_active(&self) -> bool {
        self.state != PlaybackState::Stopped
    }

    pub fn play(&mut self, title: &str, url: &str) -> bool {
        self.stop();

        let backend = match self.backend {
            Some(b) => b,
            None => return false,
        };

        let child_res = match backend {
            AudioBackend::Mpv => Command::new("mpv")
                .args(["--no-video", "--really-quiet", url])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn(),
            AudioBackend::Ffplay => Command::new("ffplay")
                .args(["-nodisp", "-autoexit", "-loglevel", "quiet", url])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn(),
            AudioBackend::Cvlc => Command::new("cvlc")
                .args(["--play-and-exit", "--no-video", "-I", "dummy", url, "vlc://quit"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn(),
            AudioBackend::Vlc => Command::new("vlc")
                .args(["--play-and-exit", "--no-video", "-I", "dummy", url, "vlc://quit"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn(),
            AudioBackend::Afplay => Command::new("afplay")
                .arg(url)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn(),
        };

        match child_res {
            Ok(child) => {
                self.child = Some(child);
                self.state = PlaybackState::Playing;
                self.current_title = Some(title.to_string());
                self.current_url = Some(url.to_string());
                true
            }
            Err(_) => {
                self.state = PlaybackState::Stopped;
                false
            }
        }
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
    }

    pub fn poll_status(&mut self) {
        if let Some(child) = &mut self.child {
            if let Ok(Some(_)) = child.try_wait() {
                self.child = None;
                self.state = PlaybackState::Stopped;
                self.current_title = None;
                self.current_url = None;
            }
        }
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        self.stop();
    }
}
