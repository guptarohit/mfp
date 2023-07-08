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

fn play_random_episodes(rss_feed: &mut Mfp, volume: u8) {
    let mut rng = thread_rng();

    if rss_feed.items.is_empty() {
        eprintln!("No Tracks found");
        return;
    }

    loop {
        if let Some(random_episode) = rss_feed.items.choose_mut(&mut rng) {
            play_episode(random_episode, volume);
            let played_title = random_episode.title.clone();
            rss_feed
                .items
                .retain(|episode| episode.title != played_title);
        } else {
            println!("All tracks have been played 🎶");
            return;
        }
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
    let args = Args::parse();

    if args.volume > 9 {
        return eprintln!("Volume must be between 0 and 9");
    }

    let mut rss_feed = Mfp::new().expect("Failed to fetch RSS data");
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
        play_random_episodes(&mut rss_feed, args.volume);
    }
}
