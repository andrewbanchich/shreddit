use clap::Parser;
use once_cell::sync::Lazy;
use reqwest::{header::HeaderMap, Client};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

mod comments;
use comments::comments;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, env = "SHREDDIT_USERNAME")]
    username: String,
    #[clap(short, long, env = "SHREDDIT_PASSWORD")]
    password: String,
    #[clap(long, env = "SHREDDIT_CLIENT_ID")]
    client_id: String,
    #[clap(long, env = "SHREDDIT_CLIENT_SECRET")]
    client_secret: String,
}

static REQWEST: Lazy<Client> = Lazy::new(|| Client::new());

#[tokio::main]
async fn main() {
    let Args {
        username,
        password,
        client_id,
        client_secret,
    } = Args::parse();

    let res = access_token(&username, &password, &client_id, &client_secret).await;

    let comms = comments(&username).await;

    for comment in comms {
        let end = if comment.data.body.len() > 20 {
            20
        } else {
            comment.data.body.len()
        };

        println!(
            "Deleting comment ({}): {}",
            comment.data.id,
            &comment.data.body[0..end]
        );
        delete_comment(&format!("t1_{}", comment.data.id), &res.access_token).await;
    }

    let comments = comments(&username).await;

    dbg!(comments);
    // me(&res.access_token).await;
    // edit_comment(&res.access_token).await;
}

#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    expires_in: u16,
    scope: String,
    token_type: String,
}

async fn access_token(
    username: &str,
    password: &str,
    client_id: &str,
    client_secret: &str,
) -> AccessTokenResponse {
    let mut params = HashMap::new();
    params.insert("grant_type", "password");
    params.insert("username", username);
    params.insert("password", password);

    REQWEST
        .post("https://www.reddit.com/api/v1/access_token")
        .form(&params)
        .basic_auth(client_id, Some(client_secret))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

async fn me(access_token: &str) {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        format!("Bearer {access_token}").parse().unwrap(),
    );

    headers.insert("User-Agent", format!("ShredditClient/0.1").parse().unwrap());

    let res: Value = REQWEST
        .get("https://oauth.reddit.com/api/v1/me")
        .headers(headers)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    dbg!(res);
}

async fn edit_comment(access_token: &str, thing_id: &str) {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        format!("Bearer {access_token}").parse().unwrap(),
    );

    headers.insert("User-Agent", format!("ShredditClient/0.1").parse().unwrap());

    let mut params = HashMap::new();
    params.insert("thing_id", thing_id);
    params.insert("text", "foo");

    let res: Value = REQWEST
        .post("https://oauth.reddit.com/api/editusertext?raw_json=1")
        .headers(headers)
        .form(&params)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    println!("{:#}", res);
}

async fn delete_comment(thing_id: &str, access_token: &str) {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        format!("Bearer {access_token}").parse().unwrap(),
    );

    headers.insert("User-Agent", format!("ShredditClient/0.1").parse().unwrap());

    let mut params = HashMap::new();
    params.insert("id", thing_id);

    let res: Value = REQWEST
        .post("https://oauth.reddit.com/api/del")
        .headers(headers)
        .form(&params)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    println!("{:#}", res);
}
