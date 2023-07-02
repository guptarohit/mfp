mod mfp;
mod mp3_stream_decoder;
mod utils;

use mfp::Mfp;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rodio::Source;
use utils::play_audio_from_url;

fn main() {
    let rss_feed = Mfp::new().expect("Failed to fetch RSS data");

    println!("MFP: Music For Programming");
    println!("Music mixes for programming and focus, unlocking the flow state.");
    println!("Streaming from: {}", rss_feed.link);

    let mut rng = thread_rng();

    if let Some(random_episode) = rss_feed.items.choose(&mut rng) {
        println!("Track name: {}", random_episode.title);
        println!("Track published date: {}", random_episode.pub_date);
        println!("Track duration: {}", random_episode.duration);

        if let Some(enclosure) = &random_episode.enclosure {
            play_audio_from_url(&enclosure.url);
        } else {
            eprint!("no episode data for randomly selected episode")
        }
    } else {
        eprint!("something went wrong while selecting random episode")
    }
}
