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

pub fn detect_backend() -> Option<AudioBackend> {
    if has_binary("mpv") {
        Some(AudioBackend::Mpv)
    } else if has_binary("ffplay") {
        Some(AudioBackend::Ffplay)
    } else if has_binary("cvlc") {
        Some(AudioBackend::Cvlc)
    } else if has_binary("vlc") {
        Some(AudioBackend::Vlc)
    } else if has_binary("afplay") {
        Some(AudioBackend::Afplay)
    } else {
        None
    }
}

pub fn has_binary(bin: &str) -> bool {
    Command::new("which")
        .arg(bin)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn spawn_player(backend: AudioBackend, url: &str, start_secs: u64) -> std::io::Result<Child> {
    let start_str = start_secs.to_string();
    let mpv_start = format!("--start={}", start_secs);
    let vlc_start = format!("--start-time={}", start_secs);

    let is_http = url.starts_with("http://") || url.starts_with("https://");
    const USER_AGENT: &str = concat!(
        "wikid/",
        env!("CARGO_PKG_VERSION"),
        " (https://github.com/sharkthakftw/wikid)"
    );
    let mpv_ua = format!("--user-agent={}", USER_AGENT);
    let vlc_ua = format!("--http-user-agent={}", USER_AGENT);

    match backend {
        AudioBackend::Mpv => {
            let mut args = vec!["--no-video", "--really-quiet"];
            if is_http {
                args.push(&mpv_ua);
            }
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
            let mut args = vec!["-nodisp", "-autoexit", "-stats", "-loglevel", "info"];
            if is_http {
                args.extend(["-user_agent", USER_AGENT]);
            }
            if start_secs > 0 {
                args.extend(["-ss", &start_str]);
            }
            args.push(url);
            Command::new("ffplay")
                .args(&args)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
        }
        AudioBackend::Cvlc => {
            let mut args = vec!["--play-and-exit", "--no-video", "-I", "dummy"];
            if is_http {
                args.push(&vlc_ua);
            }
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
            let mut args = vec!["--play-and-exit", "--no-video", "-I", "dummy"];
            if is_http {
                args.push(&vlc_ua);
            }
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
    }
}
