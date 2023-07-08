mod args;
mod mfp;
mod mp3_stream_decoder;
mod utils;

use args::Args;
use clap::Parser;
use mfp::Mfp;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rodio::Source;

use utils::parse_duration;
use utils::play_audio_from_url;

fn play_random_episode(rss_feed: &Mfp, volume: u8) {
    let mut rng = thread_rng();

    if let Some(random_episode) = rss_feed.items.choose(&mut rng) {
        play_episode(random_episode, volume);
    } else {
        eprintln!("No Tracks found");
    }
}

fn play_episode(episode: &mfp::Episode, volume: u8) {
    println!("Track name: {}", episode.title);
    println!("Track published date: {}", episode.pub_date);

    let episode_duration = parse_duration(&episode.duration).unwrap().as_secs();

    if let Some(enclosure) = &episode.enclosure {
        play_audio_from_url(&enclosure.url, volume, episode_duration);
    } else {
        eprintln!("No track data for the selected track");
    }
}

fn main() {
    let rss_feed = Mfp::new().expect("Failed to fetch RSS data");

    let args = Args::parse();

    if args.volume > 9 {
        return eprintln!("Volume must be between 0 and 9");
    }

    let total_tracks = rss_feed.items.len();

    if let Some(requested_track_number) = args.track_number {
        let requested_track_number = requested_track_number as usize;
        if requested_track_number > total_tracks || requested_track_number < 1 {
            eprintln!("Track number {requested_track_number} does not exist. Available tracks 1-{total_tracks}");
        } else {
            let episode_index = total_tracks - requested_track_number;
            let episode = &rss_feed.items[episode_index];
            play_episode(episode, args.volume);
        }
    } else {
        play_random_episode(&rss_feed, args.volume);
    }
}
