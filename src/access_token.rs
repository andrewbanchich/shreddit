use crate::cli::Config;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

pub async fn new_access_token(args: &Config, client: &Client) -> Result<String, String> {
    let params = HashMap::from([
        ("grant_type", "password"),
        ("username", &args.username),
        ("password", &args.password),
    ]);

    let res: AccessTokenResponse = client
        .post("https://www.reddit.com/api/v1/access_token")
        .form(&params)
        .basic_auth(&args.client_id, Some(&args.client_secret))
        .header("User-Agent", args.user_agent.clone())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    match res {
        AccessTokenResponse::Success { access_token } => Ok(access_token),
        AccessTokenResponse::Error { message, .. } => Err(message),
        AccessTokenResponse::Unexpected(json) => Err(serde_json::to_string(&json).unwrap()),
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AccessTokenResponse {
    Success {
        access_token: String,
    },
    Error {
        #[allow(dead_code)]
        error: String,
        message: String,
    },
    Unexpected(Value),
}
