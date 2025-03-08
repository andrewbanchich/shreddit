use super::Shred;
use crate::{cli::Config, sources::gdpr::Gdpr};
use async_trait::async_trait;
use reqwest::{Client, header::HeaderMap};
use serde::Deserialize;
use std::fmt::Debug;
use tracing::{error, info, instrument};

#[derive(Debug, Deserialize)]
pub struct Friend {
    username: String,
}

#[async_trait]
impl Shred for Friend {
    #[instrument(level = "info", skip(client, access_token))]
    async fn delete(&self, client: &Client, access_token: &str, config: &Config) {
        info!("Deleting...");

        if config.should_prevent_deletion() {
            return;
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {access_token}").parse().unwrap(),
        );

        headers.insert("User-Agent", config.user_agent.parse().unwrap());

        let uri = format!(
            "https://oauth.reddit.com/api/v1/me/friends/{}",
            self.username
        );

        let res = client.delete(&uri).headers(headers).send().await.unwrap();

        if res.status().is_success() {
            info!("Deleted!");
        } else {
            error!("Failed to delete");
        }

        self.prevent_rate_limit().await;
    }
}

impl Gdpr for Friend {
    const FILENAME: &'static str = "friends.csv";
}
