mod mfp;
mod mp3_stream_decoder;
mod utils;

use mfp::MFP;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rodio::Source;
use utils::play_audio_from_url;

fn main() {
    let rss_feed = MFP::new().expect("Failed to fetch RSS data");

    println!("Feed Title: {}", rss_feed.title);
    println!("Feed Link: {}", rss_feed.link);
    println!("Feed Description: {}", rss_feed.description);

    let mut rng = thread_rng();

    if let Some(random_episode) = rss_feed.items.choose(&mut rng) {
        println!("episode title: {}", random_episode.title);
        println!("episode pub_date: {}", random_episode.pub_date);
        println!("episode duration: {}", random_episode.duration);

        if let Some(enclosure) = &random_episode.enclosure {
            println!("episode length: {}", enclosure.length);
            println!("episode url: {}", enclosure.url);
            play_audio_from_url(&enclosure.url);
        } else {
            eprint!("no episode data for randomly selected episode")
        }
    } else {
        eprint!("something went wrong while selecting random episode")
    }
}
