use std::collections::HashMap;

use crate::{LOREM_IPSUM, REQWEST};
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

pub async fn comments(username: &str) -> Vec<CommentObj> {
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

impl Comment {
    pub async fn edit(access_token: &str, thing_id: &str) {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {access_token}").parse().unwrap(),
        );

        headers.insert("User-Agent", format!("ShredditClient/0.1").parse().unwrap());

        let mut params = HashMap::new();
        params.insert("thing_id", thing_id);
        params.insert("text", LOREM_IPSUM);

        info!("Editing comment.");

        REQWEST
            .post("https://oauth.reddit.com/api/editusertext?raw_json=1")
            .headers(headers)
            .form(&params)
            .send()
            .await
            .unwrap();
    }

    #[instrument(level = "info")]
    pub async fn delete(thing_id: &str, access_token: &str) {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {access_token}").parse().unwrap(),
        );

        headers.insert("User-Agent", format!("ShredditClient/0.1").parse().unwrap());

        let mut params = HashMap::new();
        params.insert("id", thing_id);

        info!("Deleting comment.");

        REQWEST
            .post("https://oauth.reddit.com/api/del")
            .headers(headers)
            .form(&params)
            .send()
            .await
            .unwrap();
    }
}
