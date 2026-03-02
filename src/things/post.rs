use super::Shred;
use crate::{
    cli::Config,
    sources::{api::Api, gdpr::Gdpr},
};
use async_stream::stream;
use async_trait::async_trait;
use futures_core::Stream;
use jiff::{Timestamp, Zoned, tz::TimeZone};
use reqwest::{Client, header::HeaderMap};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, error, info, instrument};

#[derive(Debug, Deserialize)]
pub struct PostData {
    data: Post,
    #[allow(dead_code)]
    kind: String,
}

#[derive(Debug, Deserialize)]
pub struct Post {
    id: String,
    #[allow(dead_code)]
    permalink: String,
    #[allow(dead_code)]
    title: String,
    subreddit: String,
    #[serde(flatten)]
    source: Source,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Source {
    Api {
        can_gild: bool,
        created_utc: f32,
        selftext: String,
        score: i64,
    },

    // GDPR cols
    // id,permalink,date,ip,subreddit,gildings,title,url,body
    Gdpr {
        #[serde(deserialize_with = "crate::sources::gdpr::date::deserialize")]
        date: Zoned,
        #[allow(dead_code)]
        ip: Option<String>,
        #[allow(dead_code)]
        gildings: Option<i64>,
        #[allow(dead_code)]
        url: Option<String>,
        #[allow(dead_code)]
        body: Option<String>,
    },
}

impl Api for Post {
    const TYPE_ID: &'static str = "t3";
}

impl Gdpr for Post {
    const FILENAME: &'static str = "posts.csv";
}

impl Post {
    /// The Reddit API uses floats for timestamps, which can't be deserialized to [`DateTime`]s. This converts the float to a datetime.
    pub fn created(&self) -> Zoned {
        match &self.source {
            Source::Api { created_utc, .. } => Timestamp::from_second(*created_utc as i64)
                .unwrap()
                .to_zoned(TimeZone::UTC),
            Source::Gdpr { date, .. } => date.clone(),
        }
    }

    fn fullname(&self) -> String {
        format!("{}_{}", Self::TYPE_ID, self.id)
    }

    fn should_skip(&self, config: &Config) -> bool {
        if let Some(skip_post_ids) = &config.skip_post_ids
            && skip_post_ids.contains(&self.id)
        {
            debug!("Skipping due to `skip_post_ids` filter");
            return true;
        }

        if let Some(skip_subreddits) = &config.skip_subreddits
            && skip_subreddits.contains(&self.subreddit)
        {
            debug!("Skipping due to `skip_subreddits` filter");
            return true;
        }

        if let Some(before) = &config.before
            && self.created().duration_since(before).as_secs() >= 0
        {
            debug!("Skipping due to `before` filter ({before})");
            return true;
        }

        if let Some(only_subreddits) = &config.only_subreddits
            && !only_subreddits.contains(&self.subreddit)
        {
            debug!("Skipping due to `only_subreddits` filter");
            return true;
        }

        match &self.source {
            Source::Api { score, .. } => {
                if let Some(max_score) = config.max_score
                    && *score > max_score
                {
                    debug!("Skipping due to `max_score` filter ({max_score})");
                    return true;
                }

                if let Some(after) = &config.after
                    && self.created().duration_since(after).as_secs() <= 0
                {
                    debug!("Skipping due to `after` filter ({after})");
                    return true;
                }
            }
            Source::Gdpr { .. } => {
                if config.max_score.is_some() {
                    error!("Cannot filter by max score when using GDPR data");
                    return true;
                }

                if let Some(before) = &config.before
                    && self.created().duration_since(before).as_secs() >= 0
                {
                    debug!("Skipping due to `before` filter ({before})");
                    return true;
                }

                if let Some(after) = &config.after
                    && self.created().duration_since(after).as_secs() <= 0
                {
                    debug!("Skipping due to `after` filter ({after})");
                    return true;
                }
            }
        }

        false
    }
}

#[async_trait]
impl Shred for Post {
    #[instrument(level = "info", skip(client, access_token))]
    async fn delete(&self, client: &Client, access_token: &str, config: &Config) {
        info!("Deleting...");

        if self.should_skip(config) || config.should_prevent_deletion() {
            return;
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {access_token}").parse().unwrap(),
        );

        headers.insert("User-Agent", config.user_agent.parse().unwrap());

        let params = HashMap::from([("id", self.fullname())]);

        let _res = client
            .post("https://oauth.reddit.com/api/del")
            .headers(headers)
            .form(&params)
            .send()
            .await
            .unwrap();

        self.prevent_rate_limit().await;
    }
}

/// https://www.reddit.com/dev/api/#GET_user_{username}_submitted
#[instrument(level = "info", skip_all)]
pub async fn list(
    client: &Client,
    access_token: &str,
    config: &Config,
) -> impl Stream<Item = Post> {
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
                format!("?after={last_seen}&limit=100")
            } else {
                "?limit=100".to_string()
            };

            let uri = format!("https://oauth.reddit.com/user/{username}/submitted.json{query_params}");

            let res: PostRes = client
                    .get(&uri)
                    .headers(headers.clone())
                    .send()
                    .await
                    .unwrap()
                    .json().await
                    .unwrap();

            match res {
                PostRes::Success { data } => {
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
                PostRes::Error(e) => {
                    error!("{e:#?}");
                    break
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PostRes {
    Success { data: PostResData },
    Error(Value),
}

#[derive(Debug, Deserialize)]
pub struct PostResData {
    pub children: Vec<PostData>,
}
