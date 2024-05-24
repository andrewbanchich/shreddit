use std::collections::HashMap;

use async_stream::stream;
use async_trait::async_trait;
use futures_core::Stream;
use reqwest::{header::HeaderMap, Client};
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, error, info, instrument};

use crate::{
    cli::Config,
    sources::{api::Api, gdpr::Gdpr},
};

use super::Shred;

#[derive(Debug, Deserialize)]
pub struct SavedPostData {
    data: SavedPost,
    #[allow(dead_code)]
    kind: String,
}

#[derive(Debug, Deserialize)]
pub struct SavedPost {
    id: String,
    subreddit: String,
    #[allow(dead_code)]
    permalink: String,
}

impl SavedPost {
    fn fullname(&self) -> String {
        format!("{}_{}", Self::TYPE_ID, self.id)
    }
}

impl Gdpr for SavedPost {
    const FILENAME: &'static str = "saved_posts.csv";
}

impl Api for SavedPost {
    const TYPE_ID: &'static str = "t3";
}

#[async_trait]
impl Shred for SavedPost {
    #[instrument(level = "info", skip(client, access_token))]
    async fn delete(&self, client: &Client, access_token: &str, config: &Config) {
        info!("Deleting...");

        if self.should_skip(config) {
            return;
        }

        if config.should_prevent_deletion() {
            return;
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {access_token}").parse().unwrap(),
        );

        headers.insert("User-Agent", config.user_agent.parse().unwrap());

        let params = HashMap::from([("id", format!("{}_{}", Self::TYPE_ID, self.id))]);

        let res = client
            .post("https://oauth.reddit.com/api/unsave")
            .headers(headers)
            .form(&params)
            .header("User-Agent", config.user_agent.clone())
            .send()
            .await
            .unwrap();

        if !res.status().is_success() {
            error!("{:#?}", res.status());
        }

        self.prevent_rate_limit().await;
    }
}

impl SavedPost {
    fn should_skip(&self, config: &Config) -> bool {
        if let Some(skip_subreddits) = &config.skip_subreddits {
            if skip_subreddits.contains(&self.subreddit) {
                debug!("Skipping due to `skip_subreddits` filter");
                return true;
            }
        }
        false
    }
}

/// https://www.reddit.com/dev/api/#GET_user_{username}_saved
#[instrument(level = "info", skip_all)]
pub async fn list(
    client: &Client,
    access_token: &str,
    config: &Config,
) -> impl Stream<Item = SavedPost> {
    info!("Fetching posts...");

    let client = client.clone();
    let username = config.username.clone();
    let user_agent = config.user_agent.clone();
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        format!("Bearer {access_token}").parse().unwrap(),
    );

    headers.insert("User-Agent", user_agent.parse().unwrap());

    stream! {
    let mut last_seen = None;

        loop {
    let query_params = if let Some(last_seen) = last_seen {
        format!("&after={last_seen}")
    } else {
        String::new()
    };

    let uri = format!("https://oauth.reddit.com/user/{username}/saved.json?&type=links{query_params}");

            let res: SavedPostRes = client
        .get(&uri)
        .headers(headers.clone())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    match res {
        SavedPostRes::Success { data } => {

    let results_len = data.children.len();

    debug!("Page contained {results_len} results");

    if results_len == 0 {
                break;
    } else {
                last_seen = data.children.last().map(|t| t.data.fullname());
    }

    for comment in data.children {
                yield comment.data;
    }
        }
        SavedPostRes::Error(e) => {
    error!("{e:#?}");
    break
        }

    }

        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SavedPostRes {
    Success { data: SavedPostResData },
    Error(Value),
}

#[derive(Debug, Deserialize)]
pub struct SavedPostResData {
    pub children: Vec<SavedPostData>,
}
