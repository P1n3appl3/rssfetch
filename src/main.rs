use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
    time::Duration,
};

use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use feed_rs::model::Entry;
use futures::future::join_all;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::runtime;

fn main() -> Result<()> {
    let f = File::open(
        env::args().nth(1).ok_or_else(|| anyhow!("pass a single filename..."))?,
    )?;
    let blogs: Vec<Blog> = BufReader::new(f)
        .lines()
        .filter_map(|s| serde_json::from_str(&s.unwrap()).ok())
        .collect();

    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let runtime = runtime::Builder::new_current_thread().enable_all().build()?;

    let posts =
        runtime.block_on(join_all(blogs.iter().map(|b: &Blog| b.get_posts(&client))));
    posts
        .into_iter()
        .flatten()
        .filter(|p| p.date > NaiveDate::from_yo_opt(2015, 1).unwrap())
        .filter_map(|p| serde_json::to_string(&p).ok())
        .for_each(|p| println!("{p}"));
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Blog {
    title: String,
    url: String,
    feed: String,
}

impl Blog {
    async fn get_posts(&self, client: &Client) -> Vec<Post> {
        macro_rules! unwrap_or_skip {
            ($x:expr) => {
                match $x {
                    Ok(x) => x,
                    Err(e) => {
                        eprintln!("❌ {}: {:?}", self.title, e);
                        return vec![];
                    }
                }
            };
        }
        let url = if self.feed.starts_with('/') {
            format!("{}{}", self.url, self.feed)
        } else {
            self.feed.clone()
        };
        let feed = unwrap_or_skip!(client.get(url).send().await);
        let bytes = unwrap_or_skip!(feed.bytes().await);
        let feed = unwrap_or_skip!(feed_rs::parser::parse(&*bytes));
        eprintln!("✅ {}: {} posts", self.title, feed.entries.len());
        feed.entries
            .into_iter()
            .filter_map(|e| match Post::from_entry(e, self) {
                Ok(post) => Some(post),
                Err(e) => {
                    eprintln!("    Missing {e:?}");
                    None
                }
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize)]
struct Posts {
    posts: Vec<Post>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Post {
    title: String,
    url: String,
    blog_title: String,
    blog_url: String,
    #[serde(with = "date_format")]
    date: NaiveDate,
}

#[derive(Copy, Clone, Debug)]
enum PostError {
    Title,
    Date,
    Link,
}

impl Post {
    fn from_entry(e: Entry, b: &Blog) -> Result<Self, PostError> {
        let mut url = match &e.links[..] {
            [] => return Err(PostError::Link),
            [single] => &single.href,
            many @ [..] => {
                // Blogger seems to stick the actual article link in the "alternate" link
                &many
                    .iter()
                    .find(|l| l.rel == Some("alternate".to_owned()))
                    .ok_or(PostError::Link)?
                    .href
            }
        }
        .to_owned();
        // Matt keeter uses relative links here... I'll humor him
        if !url.starts_with("http") {
            url = b.url.clone() + "/" + &url;
        }
        Ok(Post {
            title: e.title.ok_or(PostError::Title)?.content,
            url,
            blog_title: b.title.clone(),
            blog_url: b.url.clone(),
            date: e.published.or(e.updated).ok_or(PostError::Date)?.date_naive(),
        })
    }
}

pub mod date_format {
    use chrono::NaiveDate;
    use serde::{Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d";
    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }
}
