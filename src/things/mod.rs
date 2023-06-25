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
use std::{fmt::Debug, str::FromStr, time::Duration};
use tokio::time::sleep;
use tracing::instrument;

use crate::cli::Config;
use async_trait::async_trait;

#[async_trait]
pub trait Shred {
    async fn delete(&self, client: &Client, access_token: &str, config: &Config);
    async fn edit(&self, _client: &Client, _access_token: &str, _config: &Config) {}
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
    sleep(Duration::from_secs(2)).await; // Reddit has a rate limit

    thing.delete(client, access_token, config).await;
}

pub static LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

#[derive(Debug)]
pub enum ShredditError {
    #[allow(dead_code)]
    RateLimited,
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
