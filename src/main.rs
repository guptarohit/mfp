mod mfp;
mod mp3_stream_decoder;
mod utils;

use mfp::Mfp;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rodio::Source;
use std::env;
use utils::play_audio_from_url;

fn play_random_episode(rss_feed: &Mfp) {
    let mut rng = thread_rng();

    if let Some(random_episode) = rss_feed.items.choose(&mut rng) {
        play_episode(random_episode);
    } else {
        eprintln!("No Tracks found");
    }
}

fn play_episode(episode: &mfp::Episode) {
    println!("Track name: {}", episode.title);
    println!("Track published date: {}", episode.pub_date);
    println!("Track duration: {}", episode.duration);

    if let Some(enclosure) = &episode.enclosure {
        play_audio_from_url(&enclosure.url);
    } else {
        eprintln!("No track data for the selected track");
    }
}

fn main() {
    let rss_feed = Mfp::new().expect("Failed to fetch RSS data");

    println!("MFP: Music For Programming");
    println!("Music mixes for programming and focus, unlocking the flow state.");
    println!("Streaming from: {}", rss_feed.link);

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let total_tracks = rss_feed.items.len();
        let requested_track_number = args[1].parse::<usize>().unwrap_or_else(|_| panic!("Invalid track number. Available tracks 1-{total_tracks}"));
        if requested_track_number > total_tracks || requested_track_number < 1 {
            eprintln!("Track number {requested_track_number} does not exist. Available tracks 1-{total_tracks}");
        } else {
            let episode_index = rss_feed.items.len() - requested_track_number;
            let episode = &rss_feed.items[episode_index];
            play_episode(episode);
        }
    } else {
        println!("playing random track..");
        play_random_episode(&rss_feed);
    }
}
