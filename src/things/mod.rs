pub mod friend;
pub use friend::*;

pub mod comment;
pub use comment::*;

pub mod post;
pub use post::*;

pub mod saved_post;
pub use saved_post::*;

pub mod saved_comment;
pub use saved_comment::*;

use clap::ValueEnum;
use reqwest::Client;
use serde::Deserialize;
use std::{collections::HashSet, fmt::Debug, ops::Deref, str::FromStr, time::Duration};
use tokio::time::sleep;
use tracing::{debug, instrument};

use crate::cli::Config;
use async_trait::async_trait;

// Reddit has a new rate limit as of 7/1/2023:
// OAuth for authentication: 100 queries per minute per OAuth client id - sleep atleast 0.6s after every call ( 650 ms)
// not using OAuth for authentication: 10 queries per minute - must sleep atleast 6s between calls ( 6500 ms)
const SLEEP_DUR: Duration = Duration::from_millis(2_000); // 2s seems to be the magic number pre-7/1....
pub async fn prevent_rate_limit() {
    debug!("Sleeping for {SLEEP_DUR:?} to prevent rate limiting.");
    sleep(SLEEP_DUR).await;
}

#[async_trait]
pub trait Shred {
    async fn delete(&self, client: &Client, access_token: &str, config: &Config);
    async fn edit(&self, _client: &Client, _access_token: &str, _config: &Config) {}
    async fn prevent_rate_limit(&self) {
        prevent_rate_limit().await;
    }
    async fn shred(&self, client: &Client, access_token: &str, config: &Config) {
        self.edit(client, access_token, config).await;
        self.delete(client, access_token, config).await;
    }
}

#[instrument(level = "debug", skip(config, client, access_token))]
pub async fn shred<T>(thing: T, config: &Config, client: &Client, access_token: &str)
where
    T: Shred + Sync + Debug,
{
    thing.edit(client, access_token, config).await;
    prevent_rate_limit().await;

    thing.delete(client, access_token, config).await;
}

pub static LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

#[derive(Debug)]
pub enum ShredditError {
    #[allow(dead_code)]
    RateLimited,
    Unknown,
}

#[derive(Debug, Deserialize, PartialEq, Clone, ValueEnum)]
pub enum ThingType {
    Posts,
    Comments,
    Friends,
    SavedPosts,
    SavedComments,
}

impl FromStr for ThingType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "posts" => Ok(Self::Posts),
            "comments" => Ok(Self::Comments),
            "friends" => Ok(Self::Friends),
            "saved-posts" => Ok(Self::SavedPosts),
            "saved-comments" => Ok(Self::SavedComments),
            _ => Err("Invalid type"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubredditSet(HashSet<String>);

impl std::convert::From<&str> for SubredditSet {
    fn from(s: &str) -> Self {
        let mut subreddits = HashSet::<String>::new();
        s.split(',').for_each(|f| {
            subreddits.insert(f.to_owned());
        });
        SubredditSet(subreddits)
    }
}

impl Deref for SubredditSet {
    type Target = HashSet<String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
