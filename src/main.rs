use chrono::prelude::*;
use dotenv;
use egg_mode::{auth, tweet, KeyPair, Token};
use rss::Channel;

use std::time::Duration;

type Date = DateTime<FixedOffset>;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct Post {
    title: String,
    url: String,
    date: Date,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let token = get_token().await?;
    verify_tokens(&token, false).await?;

    loop {
        let post = last_post(&dotenv::var("RSS")?).await?;
        let last_tweet_date = last_tweet(&token).await?;

        // check if last blog post is newer than last tweet
        if post.date > last_tweet_date {
            publish(&token, &post, false).await?;
        }

        // sleep for 2 hours
        tokio::time::delay_for(Duration::from_secs(2 * 60 * 60)).await
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

async fn last_tweet(t: &Token) -> Result<Date> {
    let user_id = dotenv::var("USER_ID")?;
    let tl = egg_mode::tweet::user_timeline(user_id, false, false, &t);
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

async fn get_token() -> Result<Token> {
    let consumer = KeyPair::new(
        dotenv::var("CONSUMER_KEY")?,
        dotenv::var("CONSUMER_SECRET")?,
    );
    let access = KeyPair::new(
        dotenv::var("ACCESS_TOKEN")?,
        dotenv::var("ACCESS_TOKEN_SECRET")?,
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
