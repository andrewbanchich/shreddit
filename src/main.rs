use clap::Parser;
use once_cell::sync::Lazy;
use reqwest::{header::HeaderMap, Client};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod comments;
use comments::comments;

use crate::comments::Comment;

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
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let format = fmt::layer().with_target(false);

    tracing_subscriber::registry()
        .with(filter)
        .with(format)
        .init();

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

        Comment::edit(&res.access_token, &format!("t1_{}", comment.data.id)).await;

        info!(
            "Deleting comment ({}): {}",
            comment.data.id,
            &comment.data.body[0..end]
        );

        Comment::delete(&format!("t1_{}", comment.data.id), &res.access_token).await;
    }
}

#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    // expires_in: String,
    // scope: String,
    // token_type: String,
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

#[allow(unused)]
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

static LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
