use clap::Parser;

const ABOUT: &str = "mfp: music for programming

Music mixes (from musicforprogramming.net) for programming & focus, unlocking the flow state!

GitHub: https://github.com/guptarohit/mfp";

#[derive(Parser, Debug)]
#[clap(author, version, about = ABOUT)]
pub struct Args {
    /// Track Number, between 1 and ~68
    #[clap(short, long)]
    pub track_number: Option<u16>,

    /// Volume, between 0 and 9
    #[clap(short, long, default_value_t = 9)]
    pub volume: u8,
}
