use std::process::{Child, Command, Stdio};
use std::time::Instant;

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
    pub elapsed_secs: u64,
    pub total_duration_secs: Option<u64>,
    pub last_tick: Option<Instant>,
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
            elapsed_secs: 0,
            total_duration_secs: None,
            last_tick: None,
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

        let start_str = start_secs.to_string();
        let mpv_start = format!("--start={}", start_secs);
        let vlc_start = format!("--start-time={}", start_secs);

        const USER_AGENT: &str = "wikid/2.6.0 (https://github.com/sharkthakftw/wikid)";
        let mpv_ua = format!("--user-agent={}", USER_AGENT);
        let vlc_ua = format!("--http-user-agent={}", USER_AGENT);

        let child_res = match backend {
            AudioBackend::Mpv => {
                let mut args = vec!["--no-video", "--really-quiet", &mpv_ua];
                if start_secs > 0 {
                    args.push(&mpv_start);
                }
                args.push(url);
                Command::new("mpv")
                    .args(&args)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
            }
            AudioBackend::Ffplay => {
                let mut args = vec![
                    "-nodisp",
                    "-autoexit",
                    "-loglevel",
                    "error",
                    "-user_agent",
                    USER_AGENT,
                ];
                if start_secs > 0 {
                    args.extend(["-ss", &start_str]);
                }
                args.push(url);
                Command::new("ffplay")
                    .args(&args)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
            }
            AudioBackend::Cvlc => {
                let mut args = vec!["--play-and-exit", "--no-video", "-I", "dummy", &vlc_ua];
                if start_secs > 0 {
                    args.push(&vlc_start);
                }
                args.extend([url, "vlc://quit"]);
                Command::new("cvlc")
                    .args(&args)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
            }
            AudioBackend::Vlc => {
                let mut args = vec!["--play-and-exit", "--no-video", "-I", "dummy", &vlc_ua];
                if start_secs > 0 {
                    args.push(&vlc_start);
                }
                args.extend([url, "vlc://quit"]);
                Command::new("vlc")
                    .args(&args)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
            }
            AudioBackend::Afplay => {
                let mut args = Vec::new();
                if start_secs > 0 {
                    args.extend(["-t", &start_str]);
                }
                args.push(url);
                Command::new("afplay")
                    .args(&args)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
            }
        };

        match child_res {
            Ok(child) => {
                self.child = Some(child);
                self.state = PlaybackState::Playing;
                self.current_title = Some(title.to_string());
                self.current_url = Some(url.to_string());
                self.elapsed_secs = start_secs;
                self.total_duration_secs = total_duration_secs;
                self.last_tick = Some(Instant::now());
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
    }

    pub fn poll_status(&mut self) {
        if self.state == PlaybackState::Playing {
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
            }
        }
    }
}

pub fn parse_duration_to_secs(dur_str: &str) -> Option<u64> {
    let clean: String = dur_str
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ':' || c == '.' {
                c.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect();
    let s = clean.trim();
    if s.is_empty() {
        return None;
    }

    for token in s.split_whitespace() {
        if token.contains(':') {
            let parts: Vec<&str> = token.split(':').collect();
            match parts.len() {
                2 => {
                    if let (Ok(m), Ok(sec)) =
                        (parts[0].trim().parse::<u64>(), parts[1].trim().parse::<u64>())
                    {
                        return Some(m * 60 + sec);
                    }
                }
                3 => {
                    if let (Ok(h), Ok(m), Ok(sec)) = (
                        parts[0].trim().parse::<u64>(),
                        parts[1].trim().parse::<u64>(),
                        parts[2].trim().parse::<u64>(),
                    ) {
                        return Some(h * 3600 + m * 60 + sec);
                    }
                }
                _ => {}
            }
        }
    }

    let mut total_secs = 0u64;
    let mut found_any = false;

    let words: Vec<&str> = s.split_whitespace().collect();
    let mut i = 0;
    while i < words.len() {
        let w = words[i];
        if let Ok(num) = w.parse::<u64>() {
            if i + 1 < words.len() {
                let unit = words[i + 1];
                if unit.starts_with("hour") || unit.starts_with("hr") || unit == "h" {
                    total_secs += num * 3600;
                    found_any = true;
                    i += 2;
                    continue;
                } else if unit.starts_with("minute") || unit.starts_with("min") || unit == "m" {
                    total_secs += num * 60;
                    found_any = true;
                    i += 2;
                    continue;
                } else if unit.starts_with("second") || unit.starts_with("sec") || unit == "s" {
                    total_secs += num;
                    found_any = true;
                    i += 2;
                    continue;
                }
            }
        } else {
            if let Some(pos) = w.find(|c: char| c.is_alphabetic()) {
                let (digits, unit) = w.split_at(pos);
                if let Ok(num) = digits.parse::<u64>() {
                    if unit.starts_with('h') {
                        total_secs += num * 3600;
                        found_any = true;
                    } else if unit.starts_with('m') {
                        total_secs += num * 60;
                        found_any = true;
                    } else if unit.starts_with('s') {
                        total_secs += num;
                        found_any = true;
                    }
                }
            }
        }
        i += 1;
    }

    if found_any && total_secs > 0 {
        Some(total_secs)
    } else {
        None
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        self.stop();
    }
}
