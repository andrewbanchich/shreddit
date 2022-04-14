use std::collections::HashMap;

use crate::{ARGS, LOREM_IPSUM, REQWEST};
use reqwest::header::HeaderMap;
use serde::Deserialize;
use tracing::{info, instrument};

#[derive(Debug, Deserialize)]
struct CommentRes {
    data: Data,
}

#[derive(Debug, Deserialize)]
struct Data {
    children: Vec<CommentObj>,
}

#[derive(Debug, Deserialize)]
pub struct CommentObj {
    pub data: Comment,
}

#[derive(Debug, Deserialize)]
pub struct Comment {
    pub id: String,
    pub author: String,
    pub body: String,
    pub permalink: String,
    pub link_permalink: String,
    pub link_title: String,
}

pub async fn comments() -> Vec<CommentObj> {
    let username = &ARGS.username;

    let res: CommentRes = REQWEST
        .get(&format!("https://reddit.com/user/{username}/comments.json"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    res.data.children
}

impl CommentObj {
    pub fn preview(&self) -> &str {
        let end = if self.data.body.len() > 20 {
            20
        } else {
            self.data.body.len()
        };

        &self.data.body[..end]
    }

    #[instrument(level = "info", skip(access_token))]
    pub async fn edit(access_token: &str, thing_id: &str) {
        info!("Replacing comment with: {}", LOREM_IPSUM);

        if ARGS.dry_run {
            return;
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {access_token}").parse().unwrap(),
        );

        headers.insert("User-Agent", format!("ShredditClient/0.1").parse().unwrap());

        let mut params = HashMap::new();
        params.insert("thing_id", thing_id);
        params.insert("text", LOREM_IPSUM);

        REQWEST
            .post("https://oauth.reddit.com/api/editusertext?raw_json=1")
            .headers(headers)
            .form(&params)
            .send()
            .await
            .unwrap();
    }

    #[instrument(level = "info", skip(self, access_token))]
    pub async fn delete(&self, thing_id: &str, access_token: &str) {
        info!("Deleting comment: {}", self.preview());

        if ARGS.dry_run {
            return;
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {access_token}").parse().unwrap(),
        );

        headers.insert("User-Agent", format!("ShredditClient/0.1").parse().unwrap());

        let mut params = HashMap::new();
        params.insert("id", thing_id);

        REQWEST
            .post("https://oauth.reddit.com/api/del")
            .headers(headers)
            .form(&params)
            .send()
            .await
            .unwrap();
    }
}
