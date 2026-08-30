use std::io::Write;
use std::process::{Command, Stdio};

fn try_pipe_to_command(cmd: &str, args: &[&str], text: &str) -> bool {
    if let Ok(mut child) = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        if let Ok(status) = child.wait() {
            return status.success();
        }
    }
    false
}

pub fn copy_to_clipboard(text: &str) -> bool {
    const CANDIDATES: &[(&str, &[&str])] = &[
        ("wl-copy", &[]),
        ("xclip", &["-selection", "clipboard"]),
        ("xsel", &["--clipboard", "--input"]),
        ("pbcopy", &[]),
        ("clip.exe", &[]),
    ];

    for &(cmd, args) in CANDIDATES {
        if try_pipe_to_command(cmd, args, text) {
            return true;
        }
    }

    false
}
