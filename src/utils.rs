use reqwest::blocking::get;
use rodio::OutputStream;
use std::fmt::Write;

use crate::mp3_stream_decoder::Mp3StreamDecoder;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::time::{Duration, Instant};

/// Result of audio playback — whether the track finished naturally or the user quit.
pub enum PlaybackResult {
    /// Track played to completion.
    Finished,
    /// User pressed 'q' to stop playback.
    Quit,
}

/// Guard that disables raw mode when dropped, ensuring the terminal is restored
/// even on panic or early return.
struct RawModeGuard;

impl RawModeGuard {
    fn enable() -> Result<Self, std::io::Error> {
        enable_raw_mode()?;
        Ok(RawModeGuard)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

pub fn fetch_rss_data(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = get(url)?;
    let body = response.text()?;
    Ok(body)
}

pub fn play_audio_from_url(url: &str, volume: u8, audio_duration_sec: u64) -> PlaybackResult {
    let http_response = get(url).expect("Failed to fetch audio file");
    let source = Mp3StreamDecoder::new(http_response).unwrap();
    let (_stream, stream_handle) =
        OutputStream::try_default().expect("Failed to create audio stream");

    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    sink.append(source);
    sink.set_volume(volume as f32 / 9_f32);

    let start_time = Instant::now();
    let mut total_paused_duration = Duration::ZERO;
    let mut pause_started_at: Option<Instant> = None;

    let progress_bar_style = ProgressStyle::with_template("{wide_bar} {msg:>8} {progress_info}")
        .unwrap()
        .with_key(
            "progress_info",
            |state: &ProgressState, write: &mut dyn Write| {
                let progress_info = get_progress_bar_progress_info(state.pos(), state.len());
                write!(write, "{progress_info}").unwrap();
            },
        );
    let progress_bar = ProgressBar::new(audio_duration_sec).with_style(progress_bar_style);

    // Print controls hint before entering raw mode so \n works normally.
    println!("Controls: [space] pause/resume  [q] stop");

    // Try to enable raw mode for key event capture. If it fails (e.g. no TTY,
    // CI, piped output), fall back to non-interactive playback.
    let raw_guard = RawModeGuard::enable().ok();
    let interactive = raw_guard.is_some();

    let mut result = PlaybackResult::Finished;

    while !sink.empty() {
        // Calculate effective elapsed time (excluding paused periods)
        let effective_elapsed = if let Some(paused_at) = pause_started_at {
            // Currently paused — freeze at the moment we paused
            paused_at.duration_since(start_time) - total_paused_duration
        } else {
            start_time.elapsed() - total_paused_duration
        };
        let elapsed_seconds = effective_elapsed.as_secs();

        progress_bar.set_position(elapsed_seconds);

        if elapsed_seconds >= audio_duration_sec {
            break;
        }

        if interactive {
            // Poll for key events with a short timeout (serves as the loop sleep too)
            if event::poll(Duration::from_millis(200)).unwrap_or(false) {
                if let Ok(Event::Key(key_event)) = event::read() {
                    // Only handle key press events (not release/repeat)
                    if key_event.kind == KeyEventKind::Press {
                        match key_event.code {
                            KeyCode::Char(' ') => {
                                if sink.is_paused() {
                                    // Resuming — accumulate the time spent paused
                                    if let Some(paused_at) = pause_started_at.take() {
                                        total_paused_duration += paused_at.elapsed();
                                    }
                                    sink.play();
                                    progress_bar.set_message("");
                                } else {
                                    // Pausing — record when we paused
                                    pause_started_at = Some(Instant::now());
                                    sink.pause();
                                    progress_bar.set_message("[PAUSED]");
                                }
                            }
                            KeyCode::Char('q') => {
                                sink.stop();
                                result = PlaybackResult::Quit;
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
        } else {
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    progress_bar.finish_and_clear();
    // raw_guard is dropped here (if set), restoring the terminal
    drop(raw_guard);

    result
}

pub fn parse_duration(s: &str) -> Option<Duration> {
    let parts: Vec<_> = s.split(':').collect();
    match parts.len() {
        3 => {
            let hours = parts[0].parse::<u64>().ok()?;
            let minutes = parts[1].parse::<u64>().ok()?;
            let seconds = parts[2].parse::<u64>().ok()?;
            Duration::new(hours * 3600 + minutes * 60 + seconds, 0).into()
        }
        _ => Duration::new(0, 0).into(),
    }
}

fn get_progress_bar_progress_info(elapsed_seconds: u64, total_seconds: Option<u64>) -> String {
    let humanized_elapsed_duration = humanize_seconds_to_hours_minutes_and_seconds(elapsed_seconds);

    if let Some(total_seconds) = total_seconds {
        if total_seconds != u64::MAX {
            let humanized_total_duration =
                humanize_seconds_to_hours_minutes_and_seconds(total_seconds);
            return format!("{humanized_elapsed_duration} / {humanized_total_duration}");
        }
    }

    humanized_elapsed_duration
}

pub fn humanize_seconds_to_hours_minutes_and_seconds(seconds: u64) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        (seconds / 60) % 60,
        seconds % 60
    )
}
