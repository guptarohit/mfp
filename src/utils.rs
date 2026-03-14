use reqwest::blocking::{get, Client};
use rodio::OutputStream;
use std::fmt::Write;

use crate::mp3_stream_decoder::Mp3StreamDecoder;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::time::{Duration, Instant};

/// Number of seconds to skip forward or backward when seeking.
const SEEK_STEP_SECS: u64 = 10;

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

/// Calculate the elapsed seconds within a playback segment, excluding paused time.
fn segment_elapsed_secs(
    segment_start: Instant,
    pause_started_at: Option<Instant>,
    total_paused: Duration,
) -> u64 {
    if let Some(paused_at) = pause_started_at {
        (paused_at.duration_since(segment_start) - total_paused).as_secs()
    } else {
        (segment_start.elapsed() - total_paused).as_secs()
    }
}

pub fn fetch_rss_data(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = get(url)?;
    let body = response.text()?;
    Ok(body)
}

/// Fetch the audio stream starting at `byte_offset` using an HTTP Range request,
/// create a decoder and sink, and return the sink ready for playback.
fn start_playback_from_offset(
    client: &Client,
    url: &str,
    stream_handle: &rodio::OutputStreamHandle,
    volume: u8,
    byte_offset: u64,
) -> Result<rodio::Sink, Box<dyn std::error::Error>> {
    let response = if byte_offset == 0 {
        client.get(url).send()?
    } else {
        client
            .get(url)
            .header("Range", format!("bytes={byte_offset}-"))
            .send()?
    };

    let source = Mp3StreamDecoder::new(response)
        .map_err(|_| "Failed to create MP3 decoder from HTTP response")?;

    let sink = rodio::Sink::try_new(stream_handle)?;
    sink.append(source);
    sink.set_volume(volume as f32 / 9_f32);

    Ok(sink)
}

pub fn play_audio_from_url(
    url: &str,
    volume: u8,
    audio_duration_sec: u64,
    file_size_bytes: u64,
) -> PlaybackResult {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");
    let (_stream, stream_handle) =
        OutputStream::try_default().expect("Failed to create audio stream");

    let mut current_volume = volume;
    let mut sink = start_playback_from_offset(&client, url, &stream_handle, current_volume, 0)
        .expect("Failed to start audio playback");

    // Estimate average bytes per second for seeking via Range requests.
    // Falls back to 0 (seek disabled) if file size or duration is unknown.
    let bytes_per_sec = if audio_duration_sec > 0 && file_size_bytes > 0 {
        file_size_bytes / audio_duration_sec
    } else {
        0
    };

    // Track the logical position in the track (in seconds). This is updated
    // by wall-clock time during normal playback and directly set on seek.
    let mut current_position_secs: u64 = 0;
    let mut segment_start_time = Instant::now();
    let mut total_paused_duration = Duration::ZERO;
    let mut pause_started_at: Option<Instant> = None;

    let progress_bar_style = ProgressStyle::with_template("{wide_bar} {msg:>9} {progress_info}")
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
    if bytes_per_sec > 0 {
        println!("Controls: [space] pause/resume  [\u{2190}/h] -10s  [\u{2192}/l] +10s  [+/-] volume  [q] stop");
    } else {
        println!("Controls: [space] pause/resume  [+/-] volume  [q] stop");
    }

    // Try to enable raw mode for key event capture. If it fails (e.g. no TTY,
    // CI, piped output), fall back to non-interactive playback.
    let raw_guard = RawModeGuard::enable().ok();
    let interactive = raw_guard.is_some();

    let mut result = PlaybackResult::Finished;

    loop {
        if sink.empty() {
            break;
        }

        // Calculate effective elapsed time within the current playback segment
        // (excluding paused periods), then add to the base position.
        let display_position = current_position_secs
            + segment_elapsed_secs(segment_start_time, pause_started_at, total_paused_duration);

        progress_bar.set_position(display_position.min(audio_duration_sec));

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
                            KeyCode::Char('c')
                                if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                sink.stop();
                                result = PlaybackResult::Quit;
                                break;
                            }
                            KeyCode::Right | KeyCode::Char('l') if bytes_per_sec > 0 => {
                                // Seek forward: compute new position
                                let seg_secs = segment_elapsed_secs(
                                    segment_start_time,
                                    pause_started_at,
                                    total_paused_duration,
                                );
                                let new_pos = (current_position_secs + seg_secs + SEEK_STEP_SECS)
                                    .min(audio_duration_sec);

                                if new_pos >= audio_duration_sec {
                                    break;
                                }

                                let byte_offset = new_pos * bytes_per_sec;
                                // Create the new sink before stopping the old one so that
                                // playback continues uninterrupted if the Range request fails.
                                match start_playback_from_offset(
                                    &client,
                                    url,
                                    &stream_handle,
                                    current_volume,
                                    byte_offset,
                                ) {
                                    Ok(new_sink) => {
                                        sink.stop();
                                        sink = new_sink;
                                        current_position_secs = new_pos;
                                        segment_start_time = Instant::now();
                                        total_paused_duration = Duration::ZERO;
                                        pause_started_at = None;
                                        progress_bar.set_message("");
                                    }
                                    Err(_) => {
                                        // Seek failed — keep playing from the current position
                                    }
                                }
                            }
                            KeyCode::Left | KeyCode::Char('h') if bytes_per_sec > 0 => {
                                // Seek backward: compute new position
                                let seg_secs = segment_elapsed_secs(
                                    segment_start_time,
                                    pause_started_at,
                                    total_paused_duration,
                                );
                                let new_pos = (current_position_secs + seg_secs)
                                    .saturating_sub(SEEK_STEP_SECS);

                                let byte_offset = new_pos * bytes_per_sec;
                                // Create the new sink before stopping the old one so that
                                // playback continues uninterrupted if the Range request fails.
                                match start_playback_from_offset(
                                    &client,
                                    url,
                                    &stream_handle,
                                    current_volume,
                                    byte_offset,
                                ) {
                                    Ok(new_sink) => {
                                        sink.stop();
                                        sink = new_sink;
                                        current_position_secs = new_pos;
                                        segment_start_time = Instant::now();
                                        total_paused_duration = Duration::ZERO;
                                        pause_started_at = None;
                                        progress_bar.set_message("");
                                    }
                                    Err(_) => {
                                        // Seek failed — keep playing from the current position
                                    }
                                }
                            }
                            KeyCode::Char('+') | KeyCode::Char('=') => {
                                if current_volume < 9 {
                                    current_volume += 1;
                                    sink.set_volume(current_volume as f32 / 9_f32);
                                    progress_bar.set_message(format!("[VOL {current_volume}/9]"));
                                }
                            }
                            KeyCode::Char('-') | KeyCode::Char('_') => {
                                if current_volume > 0 {
                                    current_volume -= 1;
                                    sink.set_volume(current_volume as f32 / 9_f32);
                                    progress_bar.set_message(format!("[VOL {current_volume}/9]"));
                                }
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
