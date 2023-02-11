use crate::cli::Config;
use reqwest::Client;
use std::collections::HashMap;

pub async fn new_access_token(args: &Config, client: &Client) -> Result<String, String> {
    let params = HashMap::from([
        ("grant_type", "password"),
        ("username", &args.username),
        ("password", &args.password),
    ]);

    let res: serde_json::Value = client
        .post("https://www.reddit.com/api/v1/access_token")
        .form(&params)
        .basic_auth(&args.client_id, Some(&args.client_secret))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    if res.get("error").is_some() {
        return Err(res["message"].as_str().unwrap().to_owned());
    } else {
        return Ok(res["access_token"].as_str().unwrap().to_owned());
    }
}
