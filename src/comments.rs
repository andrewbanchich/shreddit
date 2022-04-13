use crate::REQWEST;
use serde::Deserialize;

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
