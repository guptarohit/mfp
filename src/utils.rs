use reqwest::blocking::get;
use rodio::{Decoder, OutputStream};
use std::io::Cursor;

pub fn fetch_rss_data(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = get(url)?;
    let body = response.text()?;
    Ok(body)
}

pub fn play_audio_from_url(url: &str) {
    let http_response = reqwest::blocking::get(url).expect("Failed to fetch audio file");
    let audio_data = http_response.bytes().expect("Failed to read audio data");
    let audio_cursor = Cursor::new(audio_data);

    let decoder = Decoder::new(audio_cursor).expect("Failed to create audio decoder");

    let (_stream, stream_handle) =
        OutputStream::try_default().expect("Failed to create audio stream");

    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    sink.append(decoder);
    sink.sleep_until_end();
}
