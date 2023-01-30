use async_stream::stream;
use chrono::{DateTime, NaiveDateTime, Utc};
use clap::ValueEnum;
use futures_core::stream::Stream;
use reqwest::{header::HeaderMap, Client};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, str::FromStr};
use tracing::{debug, error, info, instrument};

#[derive(Debug)]
pub enum ShredditError {
    RateLimited,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum Thing {
    #[serde(rename = "t1")]
    Comment {
        id: String,
        body: String,
        permalink: String,
        created_utc: f32,
    },
    #[serde(rename = "t3")]
    Post {
        id: String,
        selftext: String,
        permalink: String,
        title: String,
        created_utc: f32,
    },
}

#[derive(Debug, Deserialize)]
struct ThingRes {
    data: ThingResData,
}

#[derive(Debug, Deserialize)]
pub struct ThingResData {
    pub children: Vec<Thing>,
}

#[derive(Debug, Deserialize, PartialEq, Clone, ValueEnum)]
pub enum ThingType {
    Posts,
    Comments,
}

impl FromStr for ThingType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "posts" => Ok(Self::Posts),
            "comments" => Ok(Self::Comments),
            _ => Err("Invalid type"),
        }
    }
}

static LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

impl Thing {
    pub fn created(&self) -> DateTime<Utc> {
        let timestamp = match self {
            Thing::Comment { created_utc, .. } => *created_utc,
            Thing::Post { created_utc, .. } => *created_utc,
        } as i64;

        let dt = NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();

        DateTime::from_utc(dt, Utc)
    }

    pub fn preview(s: &str) -> &str {
        let end = if s.len() > 50 { 50 } else { s.len() };
        &s[..end]
    }

    pub fn reddit_type_id(&self) -> &str {
        match self {
            Thing::Comment { .. } => "t1",
            Thing::Post { .. } => "t3",
        }
    }

    pub fn fullname(&self) -> String {
        let unique_id = match self {
            Thing::Comment { id, .. } => id,
            Thing::Post { id, .. } => id,
        };

        format!("{type_id}_{unique_id}", type_id = self.reddit_type_id())
    }

    /// https://www.reddit.com/dev/api/#GET_user_{username}_submitted
    #[instrument(level = "info", skip(client, username))]
    pub async fn list(
        client: &Client,
        thing_type: &ThingType,
        username: &str,
    ) -> impl Stream<Item = Thing> {
        let thing_type = match thing_type {
            ThingType::Comments => "comments",
            ThingType::Posts => "submitted",
        };

        info!("Fetching {thing_type}...");

        let username = username.to_owned();
        let client = client.clone();

        stream! {
            // The fullname of the last seen Thing.
            let mut last_seen = None;

            loop {
            let query_params = if let Some(last_seen) = last_seen {
        format!("?after={last_seen}")
            } else {
        String::new()
            };

        debug!("Iterating over next page of results");

        let uri = format!("https://reddit.com/user/{username}/{thing_type}.json{query_params}");

                let res: ThingRes = client
            .get(&uri)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        if res.data.children.is_empty() {
                    debug!("Completed listing {thing_type}");
                    break;
        } else {
            last_seen = res.data.children.last().map(|t| t.fullname());
        }

                for thing in res.data.children {
            yield thing;
                }
            }

            }
    }

    #[instrument(level = "debug", skip(self, client, access_token))]
    pub async fn edit(
        &self,
        client: &Client,
        access_token: &str,
        dry_run: bool,
    ) -> Result<(), ShredditError> {
        #[derive(Debug, Deserialize)]
        struct EditResponse {
            jquery: Vec<Value>,
            success: bool,
        }

        let id = match self {
            Self::Post { id, title, .. } => {
                info!("Editing post: {title}");
                format!("t3_{id}")
            }
            Self::Comment { id, body, .. } => {
                info!("Editing comment: {}", Thing::preview(body));
                format!("t1_{id}")
            }
        };

        if dry_run {
            return Ok(());
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {access_token}").parse().unwrap(),
        );

        headers.insert("User-Agent", format!("ShredditClient/0.1").parse().unwrap());

        let params = HashMap::from([("thing_id", id), ("text", LOREM_IPSUM.to_string())]);

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

        if !res.success {
            if res
                .jquery
                .iter()
                .find(|array| {
                    array
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|item| match item {
                            Value::Array(a) => a.contains(&Value::String(
                                ".error.RATELIMIT.field-ratelimit".to_string(),
                            )),
                            _ => false,
                        })
                        .is_some()
                })
                .is_some()
            {
                error!("RATE LIMITED");
                return Err(ShredditError::RateLimited);
            } else {
                error!("Couldn't edit: {res:#?}");
            }
        }

        Ok(())
    }

    #[instrument(level = "info", skip(self, client, access_token))]
    pub async fn delete(&self, client: &Client, access_token: &str, dry_run: bool) {
        let id = match self {
            Self::Post { id, title, .. } => {
                info!("Deleting post: {title}");
                format!("t3_{id}")
            }
            Self::Comment { id, body, .. } => {
                info!("Deleting comment: {}", Thing::preview(body));
                format!("t1_{id}")
            }
        };

        if dry_run {
            return;
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {access_token}").parse().unwrap(),
        );

        headers.insert("User-Agent", format!("ShredditClient/0.1").parse().unwrap());

        let params = HashMap::from([("text", LOREM_IPSUM.to_string())]);

        client
            .post("https://oauth.reddit.com/api/del")
            .headers(headers)
            .form(&params)
            .send()
            .await
            .unwrap();
    }
}
