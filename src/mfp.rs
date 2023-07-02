use rss::Channel;

use crate::utils::fetch_rss_data;

const RSS_URL: &str = "https://musicforprogramming.net/rss.xml";

// #[derive(Debug)]
pub struct Mfp {
    pub title: String,
    pub link: String,
    pub description: String,
    pub items: Vec<Episode>,
}

impl Mfp {
    pub fn new() -> Result<Mfp, Box<dyn std::error::Error>> {
        let rss_data = fetch_rss_data(RSS_URL).expect("Failed to fetch RSS data");

        let channel = Channel::read_from(rss_data.as_bytes())?;

        let items = channel
            .items()
            .iter()
            .map(|item| {
                let enclosure = item.enclosure().map(|enc| Enclosure {
                    url: enc.url().to_string(),
                    length: enc.length().parse::<u64>().unwrap_or_default(),
                    mime_type: enc.mime_type().to_string(),
                });

                let keywords = item
                    .itunes_ext()
                    .and_then(|ext| ext.keywords())
                    .unwrap_or_default()
                    .split(',')
                    .map(|keyword| keyword.trim().to_string())
                    .collect();

                Episode::new(
                    item.title().unwrap_or_default().to_string(),
                    item.link().unwrap_or_default().to_string(),
                    item.pub_date().unwrap_or_default().to_string(),
                    enclosure,
                    item.itunes_ext()
                        .and_then(|ext| ext.duration())
                        .unwrap_or_default()
                        .to_string(),
                    keywords,
                )
            })
            .collect();

        let feed = Mfp {
            title: channel.title().to_owned(),
            link: channel.link().to_owned(),
            description: channel.description().to_owned(),
            items,
        };
        Ok(feed)
    }
}

#[derive(Debug)]
pub struct Episode {
    pub title: String,
    pub link: String,
    pub pub_date: String,
    pub enclosure: Option<Enclosure>,
    pub duration: String,
    pub keywords: Vec<String>,
}

#[derive(Debug)]
pub struct Enclosure {
    pub url: String,
    pub length: u64,
    pub mime_type: String,
}

impl Episode {
    fn new(
        title: String,
        link: String,
        pub_date: String,
        enclosure: Option<Enclosure>,
        duration: String,
        keywords: Vec<String>,
    ) -> Episode {
        Episode {
            title,
            link,
            pub_date,
            enclosure,
            duration,
            keywords,
        }
    }
}
