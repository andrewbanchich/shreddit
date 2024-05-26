use super::{Shred, ShredditError};
use crate::{
    cli::Config,
    sources::{api::Api, gdpr::Gdpr},
};
use async_stream::stream;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use futures_core::Stream;
use reqwest::{header::HeaderMap, Client};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, error, info, instrument};

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct Comment {
    id: String,
    body: String,
    permalink: String,
    subreddit: String,
    #[serde(flatten)]
    source: Source,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Source {
    Api {
        score: i64,
        created_utc: f32,
        can_gild: bool,
    },

    // GDPR columns
    // id,permalink,date,ip,subreddit,gildings,link,parent,body,media
    Gdpr {
        date: DateTime<Utc>,
        subreddit: String,
    },
}

impl Api for Comment {
    const TYPE_ID: &'static str = "t1";
}

impl Gdpr for Comment {
    const FILENAME: &'static str = "comments.csv";
}

#[async_trait]
impl Shred for Comment {
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
            .header("User-Agent", config.user_agent.clone())
            .send()
            .await
            .unwrap();

        self.prevent_rate_limit().await;
    }

    #[instrument(level = "debug", skip(client, access_token))]
    async fn edit(&self, client: &Client, access_token: &str, config: &Config) {
        #[allow(unused)]
        #[derive(Debug, Deserialize)]
        #[serde(untagged)]
        enum EditResponse {
            Success { jquery: Vec<Value>, success: bool },
            Unexpected(Value),
        }

        debug!("Editing...");

        if self.should_skip(config) {
            return;
        }

        if config.dry_run {
            return;
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {access_token}").parse().unwrap(),
        );

        headers.insert("User-Agent", config.user_agent.parse().unwrap());

        let params = HashMap::from([
            ("thing_id", self.fullname()),
            ("text", config.replacement_comment.to_string()),
        ]);

        let res: EditResponse = client
            .post("https://oauth.reddit.com/api/editusertext?raw_json=1")
            .headers(headers)
            .form(&params)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        match res {
            EditResponse::Success { jquery, .. } => {
                if jquery.iter().any(|array| {
                    array.as_array().unwrap().iter().any(|item| match item {
                        Value::Array(a) => a.contains(&Value::String(
                            ".error.RATELIMIT.field-ratelimit".to_string(),
                        )),
                        _ => false,
                    })
                }) {
                    error!("RATE LIMITED");
                }
            }
            EditResponse::Unexpected(v) => match self.source {
                Source::Api { can_gild, .. } => {
                    if !can_gild {
                        error!("Couldn't edit - comment was probably removed by a moderator (`can_gild` == {})", can_gild);
                    } else {
                        error!("Couldn't edit: {v:#?}");
                    }
                }
                Source::Gdpr { .. } => {
                    let Ok(comment) = self.to_api(client, access_token, config).await else {
                        return;
                    };

                    match comment.source {
                        Source::Api { can_gild, .. } => {
                            if !can_gild {
                                error!("Couldn't edit - comment was probably removed by a moderator (`can_gild` == {})", can_gild);
                            } else {
                                error!("Couldn't edit: {v:#?}");
                            }
                        }
                        Source::Gdpr { .. } => {
                            unreachable!()
                        }
                    }
                }
            },
        };

        self.prevent_rate_limit().await;
    }
}

impl Comment {
    /// The Reddit API uses floats for timestamps, which can't be deserialized to [`DateTime`]s. This converts the float to a datetime.
    pub fn created(&self) -> DateTime<Utc> {
        match &self.source {
            Source::Api { created_utc, .. } => {
                let dt = NaiveDateTime::from_timestamp_opt(*created_utc as i64, 0).unwrap();
                Utc.from_utc_datetime(&dt)
            }
            Source::Gdpr { date, .. } => *date,
        }
    }

    fn fullname(&self) -> String {
        format!("{}_{}", Self::TYPE_ID, self.id)
    }

    fn should_skip(&self, config: &Config) -> bool {
        if let Some(skip_subreddits) = &config.skip_subreddits {
            if skip_subreddits.contains(&self.subreddit) {
                debug!("Skipping due to `skip_subreddits` filter");
                return true;
            }
        }
        if self.created() >= config.before {
            debug!("Skipping due to `before` filter ({})", config.before);
            return true;
        }
        match &self.source {
            Source::Api { score, .. } => {
                if let Some(max_score) = config.max_score {
                    if *score > max_score {
                        debug!("Skipping due to `max_score` filter ({max_score})");
                        return true;
                    }
                }
            }
            Source::Gdpr { .. } => {
                if config.max_score.is_some() {
                    error!("Cannot filter by max score when using GDPR data");
                    return true;
                }
            }
        }

        false
    }

    async fn to_api(
        &self,
        client: &Client,
        access_token: &str,
        config: &Config,
    ) -> Result<Self, ShredditError> {
        debug!("Getting comment from API...");

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {access_token}").parse().unwrap(),
        );

        headers.insert("User-Agent", config.user_agent.parse().unwrap());

        let uri = format!(
            "https://oauth.reddit.com/api/info.json?id={}",
            self.fullname()
        );

        let res: Response = client
            .get(&uri)
            .headers(headers)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        match res {
            Response::Success { data } => match data.children.into_iter().next() {
                Some(c) => Ok(c.data),
                None => {
                    error!("Couldn't get comment from API: No data returned");
                    Err(ShredditError::Unknown)
                }
            },
            Response::Error(e) => {
                error!("Couldn't get comment from API: {e:#?}");
                Err(ShredditError::Unknown)
            }
        }
    }
}

/// https://www.reddit.com/dev/api/#GET_user_{username}_submitted
#[instrument(level = "info", skip_all)]
pub async fn list(client: &Client, config: &Config) -> impl Stream<Item = Comment> {
    info!("Fetching comments...");

    let username = config.username.to_owned();
    let client = client.clone();
    let user_agent = config.user_agent.clone();

    stream! {
    let mut last_seen = None;

        loop {
    let query_params = if let Some(last_seen) = last_seen {
        format!("?after={last_seen}")
    } else {
        String::new()
    };

    let uri = format!("https://reddit.com/user/{username}/comments.json{query_params}");

            let res: Response = client
        .get(&uri)
        .header("User-Agent", user_agent.clone())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    match res {
        Response::Success { data} => {

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
        Response::Error(e) => {
    error!("Failed to fetch comments: {e:#?}");
    break
        }

    }

        }
    }
}

// #[derive(Debug, Deserialize)]
// #[serde(tag = "kind", content = "data")]
// pub enum Thing {
//     #[serde(rename = "t1")]
//     Comment {
//         id: String,
//         body: String,
//         permalink: String,
//         created_utc: f32,
//         score: i64,
//     },
//     #[serde(rename = "t3")]
//     Post {
//         id: String,
//         selftext: String,
//         permalink: String,
//         title: String,
//         created_utc: f32,
//         score: i64,
//     },
// }

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Response {
    Success { data: ResponseData },
    Error(Value),
}

#[derive(Debug, Deserialize)]
pub struct ResponseData {
    pub children: Vec<Child>,
    pub after: Option<String>,
    pub before: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Child {
    pub data: Comment,
}
