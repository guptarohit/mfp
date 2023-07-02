use reqwest::blocking::get;
use rodio::OutputStream;

use crate::mp3_stream_decoder::Mp3StreamDecoder;

pub fn fetch_rss_data(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = get(url)?;
    let body = response.text()?;
    Ok(body)
}

pub fn play_audio_from_url(url: &str, volume: u8) {
    let http_response = reqwest::blocking::get(url).expect("Failed to fetch audio file");
    let source = Mp3StreamDecoder::new(http_response).unwrap();
    let (_stream, stream_handle) =
        OutputStream::try_default().expect("Failed to create audio stream");

    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    sink.append(source);
    sink.set_volume(volume as f32 /9_f32);
    sink.sleep_until_end();
}
