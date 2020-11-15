use chrono::prelude::*;
use dotenv::dotenv;
use egg_mode::{auth, tweet, KeyPair, Token};
use rss::Channel;
use structopt::StructOpt;

use std::time::Duration;

type Date = DateTime<FixedOffset>;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, StructOpt)]
#[structopt(
    name = "twitter-bot",
    about = "A good bot that tweets automatically for every new blog post"
)]
struct Args {
    #[structopt(long)]
    user_id: Option<String>,

    #[structopt(long)]
    rss: Option<String>,

    #[structopt(long)]
    consumer_key: Option<String>,

    #[structopt(long)]
    consumer_secret: Option<String>,

    #[structopt(long)]
    access_token: Option<String>,

    #[structopt(long)]
    access_token_secret: Option<String>,

    #[structopt(long)]
    bearer_token: Option<String>,

    /// Enable a minimal logging
    #[structopt(short, long)]
    log: bool,
}

impl Args {
    fn parse() -> Self {
        let args = Self::default();
        match args.rss.is_none()
            || args.user_id.is_none()
            || args.consumer_key.is_none()
            || args.consumer_secret.is_none()
            || args.access_token.is_none()
            || args.access_token_secret.is_none()
            || args.bearer_token.is_none()
        {
            true => Self::from_args(),
            _ => args,
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        dotenv().ok();

        Self {
            user_id: dotenv::var("USER_ID").ok(),
            rss: dotenv::var("RSS").ok(),
            consumer_key: dotenv::var("CONSUMER_KEY").ok(),
            consumer_secret: dotenv::var("CONSUMER_SECRET").ok(),
            access_token: dotenv::var("ACCESS_TOKEN").ok(),
            access_token_secret: dotenv::var("ACCESS_TOKEN_SECRET").ok(),
            bearer_token: dotenv::var("BEARER_TOKEN").ok(),
            log: false,
        }
    }
}

struct Post {
    title: String,
    url: String,
    date: Date,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    dbg!(&args);
    let token = get_token(args.clone()).await?;
    verify_tokens(&token, args.log).await?;

    match (&args.rss, &args.user_id) {
        (Some(rss), Some(user_id)) => {
            loop {
                let post = last_post(rss).await?;
                let last_tweet_date = last_tweet(&token, user_id).await?;

                // check if last blog post is newer than last tweet
                if post.date > last_tweet_date {
                    publish(&token, &post, false).await?;
                }
                // sleep for 2 hours
                tokio::time::delay_for(Duration::from_secs(2 * 60 * 60)).await
            }
        }
        _ => Ok(eprintln!("Unable to parse RSS and USER_ID")),
    }
}

async fn last_post(rss: &str) -> Result<Post> {
    let content = reqwest::get(rss).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;

    let last_post = channel.items().first().ok_or("No post found")?;

    let (title, url, date) = (
        String::from(last_post.title().unwrap()),
        String::from(last_post.link().unwrap()),
        DateTime::parse_from_rfc2822(last_post.pub_date().unwrap())?,
    );

    Ok(Post { title, url, date })
}

async fn last_tweet(t: &Token, user_id: &str) -> Result<Date> {
    let tl = egg_mode::tweet::user_timeline(user_id.to_string(), false, false, &t);
    let (_, feed) = tl.start().await?;

    let tweet = feed.response.first().unwrap();
    let date = DateTime::parse_from_rfc2822(&tweet.created_at.to_rfc2822())?;

    Ok(date)
}

async fn publish(t: &Token, post: &Post, log: bool) -> Result<()> {
    let text = format!("Nuovo post - {}\n{}", post.title, post.url);
    tweet::DraftTweet::new(text.clone()).send(t).await?;
    if log {
        println!("{:#?}", text);
    }

    Ok(())
}

async fn get_token(args: Args) -> Result<Token> {
    let consumer = KeyPair::new(
        args.consumer_key.expect("Unable to parse CONSUMER_KEY"),
        args.consumer_secret
            .expect("Unable to parse CONSUMER_SECRET"),
    );
    let access = KeyPair::new(
        args.access_token.expect("Unable to parse ACCESS_TOKEN"),
        args.access_token_secret
            .expect("Unable to parse ACCESS_TOKEN_SECRET"),
    );

    Ok(Token::Access { consumer, access })
}

async fn verify_tokens(t: &Token, log: bool) -> Result<()> {
    let respose = auth::verify_tokens(t).await?;
    if log {
        println!("{:#?}", respose);
    }

    Ok(())
}
