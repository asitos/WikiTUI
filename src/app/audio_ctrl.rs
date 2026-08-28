use crate::app::pane::PaneContent;
use crate::app::App;

impl App {
    pub fn toggle_spoken_audio(&mut self) {
        if self.audio_player.is_active() {
            self.audio_player.toggle_pause();
            let msg = match self.audio_player.state {
                crate::audio::PlaybackState::Playing => "audio resumed".to_string(),
                crate::audio::PlaybackState::Paused => "audio paused".to_string(),
                crate::audio::PlaybackState::Stopped => "audio stopped".to_string(),
            };
            self.set_status_message(msg);
            return;
        }

        let track_info = match &self.active_pane().content {
            PaneContent::ArticleText {
                parsed_doc, title, ..
            } => {
                if let Some(spoken) = &parsed_doc.spoken_audio {
                    spoken
                        .tracks
                        .first()
                        .map(|t| (title.clone(), t.url.clone()))
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some((play_title, track_url)) = track_info {
            let success = self.audio_player.play(&play_title, &track_url);
            if success {
                self.set_status_message(format!("playing spoken article: {}", play_title));
            } else if self.audio_player.backend.is_none() {
                self.set_status_message(
                    "no audio backend found (install mpv, ffplay, or cvlc)".to_string(),
                );
            } else {
                self.set_status_message("failed to start audio playback".to_string());
            }
        } else {
            self.set_status_message("no spoken audio available for this article".to_string());
        }
    }

    pub fn stop_spoken_audio(&mut self) {
        if self.audio_player.is_active() {
            self.audio_player.stop();
            self.set_status_message("audio playback stopped".to_string());
        }
    }
}
