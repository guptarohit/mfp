use reqwest::blocking::get;
use rodio::OutputStream;
use std::fmt::Write;

use crate::mp3_stream_decoder::Mp3StreamDecoder;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::time::{Duration, Instant};

pub fn fetch_rss_data(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = get(url)?;
    let body = response.text()?;
    Ok(body)
}

pub fn play_audio_from_url(url: &str, volume: u8, audio_duration_sec: u64) {
    let http_response = get(url).expect("Failed to fetch audio file");
    let source = Mp3StreamDecoder::new(http_response).unwrap();
    let (_stream, stream_handle) =
        OutputStream::try_default().expect("Failed to create audio stream");

    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    sink.append(source);
    sink.set_volume(volume as f32 / 9_f32);

    let start_time = Instant::now();
    let progress_bar_style = ProgressStyle::with_template("{wide_bar} {progress_info}")
        .unwrap()
        .with_key(
            "progress_info",
            |state: &ProgressState, write: &mut dyn Write| {
                let progress_info = get_progress_bar_progress_info(state.pos(), state.len());
                write!(write, "{progress_info}").unwrap();
            },
        );
    let progress_bar = ProgressBar::new(audio_duration_sec).with_style(progress_bar_style);

    while !sink.empty() {
        let elapsed = start_time.elapsed();
        let elapsed_seconds = elapsed.as_secs();
        progress_bar.set_position(elapsed_seconds);
        if elapsed_seconds >= audio_duration_sec {
            break;
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    progress_bar.finish();
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
