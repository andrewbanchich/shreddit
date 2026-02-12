use std::collections::HashMap;

use async_stream::stream;
use async_trait::async_trait;
use futures_core::Stream;
use reqwest::{Client, header::HeaderMap};
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, error, info, instrument, warn};

use crate::{
    cli::Config,
    sources::{api::Api, gdpr::Gdpr},
};

use super::Shred;

#[derive(Debug, Deserialize)]
pub struct UpvotedData {
    data: Upvoted,
    kind: String,
}

impl UpvotedData {
    fn with_type(mut self) -> Upvoted {
        self.data.type_id = self.kind;
        self.data
    }
}

#[allow(unused)]
#[derive(Debug, Deserialize, Clone)]
pub struct Upvoted {
    id: String,
    permalink: String,
    #[serde(flatten)]
    source: Source,
    #[serde(skip)]
    type_id: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
enum Source {
    Api {
        subreddit: String,
    },
    // GDPR columns: id,permalink,direction
    // Note: We filter for direction="up" when reading
    Gdpr {},
}

// Vote struct for reading from GDPR vote CSV files
#[derive(Debug, Deserialize)]
pub struct Vote {
    pub id: String,
    pub permalink: String,
    pub direction: String,
}

impl Vote {
    pub fn into_upvoted(self, type_id: String) -> Upvoted {
        Upvoted {
            id: self.id,
            permalink: self.permalink,
            source: Source::Gdpr {},
            type_id,
        }
    }
}

impl Gdpr for Vote {
    const FILENAME: &'static str = "post_votes.csv";
}

impl Api for Upvoted {
    const TYPE_ID: &'static str = "t3";
}

impl Upvoted {
    fn fullname(&self) -> String {
        format!("{}_{}", self.type_id, self.id)
    }

    fn subreddit(&self) -> Option<&str> {
        match &self.source {
            Source::Api { subreddit } => Some(subreddit),
            Source::Gdpr {} => None,
        }
    }
}

#[async_trait]
impl Shred for Upvoted {
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

        let params = HashMap::from([
            ("id", self.fullname()),
            ("dir", "0".to_string()),
        ]);

        let res = client
            .post("https://oauth.reddit.com/api/vote")
            .headers(headers)
            .form(&params)
            .send()
            .await
            .unwrap();

        if !res.status().is_success() {
            error!("{:#?}", res.text().await.unwrap());
        }

        self.prevent_rate_limit().await;
    }
}

impl Upvoted {
    fn should_skip(&self, config: &Config) -> bool {
        if let Some(skip_comment_ids) = &config.skip_comment_ids
            && skip_comment_ids.contains(&self.id)
        {
            debug!("Skipping due to `skip_comment_ids` filter");
            return true;
        }

        // Subreddit filters only work with API data
        if let Some(subreddit) = self.subreddit() {
            if let Some(skip_subreddits) = &config.skip_subreddits
                && skip_subreddits.contains(subreddit)
            {
                debug!("Skipping due to `skip_subreddits` filter");
                return true;
            }
            if let Some(only_subreddits) = &config.only_subreddits
                && !only_subreddits.contains(subreddit)
            {
                debug!("Skipping due to `only_subreddits` filter");
                return true;
            }
        } else if config.skip_subreddits.is_some() || config.only_subreddits.is_some() {
            debug!("Cannot filter by subreddit when using GDPR data");
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
) -> impl Stream<Item = Upvoted> {
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

    let uri = format!("https://oauth.reddit.com/user/{username}/upvoted?{query_params}");

    let res = client
                .get(&uri)
                .headers(headers.clone())
                .send()
                .await
                .unwrap()
                .json()
                .await;

    let res = match res {
        Ok(res) => res,
        Err(_e) => {
            warn!("first attempt to list failed");

            let res: UpvotedRes = client
                        .get(&uri)
                        .headers(headers.clone())
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
            res
        }
    };

    match res {
        UpvotedRes::Success { data } => {

    let results_len = data.children.len();

    debug!("Page contained {results_len} results");

    if results_len == 0 {
                break;
    } else {
                last_seen = data.children.last().map(|t| {
                    let mut item = t.data.clone();
                    item.type_id = t.kind.clone();
                    item.fullname()
                });
    }

    for item in data.children {
                yield item.with_type();
    }
        }
        UpvotedRes::Error(e) => {
    error!("{e:#?}");
    break
        }

    }

        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum UpvotedRes {
    Success { data: UpvotedResData },
    Error(Value),
}

#[derive(Debug, Deserialize)]
pub struct UpvotedResData {
    pub children: Vec<UpvotedData>,
}
