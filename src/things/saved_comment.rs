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
pub struct SavedCommentData {
    data: SavedComment,
    #[allow(dead_code)]
    kind: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct SavedComment {
    id: String,
    subreddit: String,
    permalink: String,
}

impl Gdpr for SavedComment {
    const FILENAME: &'static str = "saved_comments.csv";
}

impl Api for SavedComment {
    const TYPE_ID: &'static str = "t1";
}

impl SavedComment {
    fn fullname(&self) -> String {
        format!("{}_{}", Self::TYPE_ID, self.id)
    }
}

#[async_trait]
impl Shred for SavedComment {
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

impl SavedComment {
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
) -> impl Stream<Item = SavedComment> {
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

    let uri = format!("https://oauth.reddit.com/user/{username}/saved.json?&type=comments{query_params}");

            let res: SavedCommentRes = client
        .get(&uri)
        .headers(headers.clone())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    match res {
        SavedCommentRes::Success { data } => {

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
        SavedCommentRes::Error(e) => {
    error!("{e:#?}");
    break
        }

    }

        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SavedCommentRes {
    Success { data: SavedCommentResData },
    Error(Value),
}

#[derive(Debug, Deserialize)]
pub struct SavedCommentResData {
    pub children: Vec<SavedCommentData>,
}
