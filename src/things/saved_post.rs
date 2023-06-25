use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::{header::HeaderMap, Client};
use serde::Deserialize;
use tracing::{error, info, instrument};

use crate::{
    cli::Config,
    sources::{api::Api, gdpr::Gdpr},
};

use super::Shred;

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct SavedPost {
    id: String,
    permalink: String,
}

impl Gdpr for SavedPost {
    const FILENAME: &'static str = "saved_posts.csv";
}

impl Api for SavedPost {
    const TYPE_ID: &'static str = "t3";
}

#[async_trait]
impl Shred for SavedPost {
    #[instrument(level = "info", skip(client, access_token))]
    async fn delete(&self, client: &Client, access_token: &str, config: &Config) {
        info!("Deleting...");

        // if self.should_skip(config) {
        //     return;
        // }

        if config.dry_run {
            return;
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {access_token}").parse().unwrap(),
        );

        headers.insert("User-Agent", config.user_agent.parse().unwrap());

        let params = HashMap::from([("id", format!("{}_{}", Self::TYPE_ID, self.id))]);

        let res = client
            .post("https://oauth.reddit.com/api/unsave")
            .headers(headers)
            .form(&params)
            .header("User-Agent", config.user_agent.clone())
            .send()
            .await
            .unwrap();

        if !res.status().is_success() {
            error!("{:#?}", res.status());
        }

        self.prevent_rate_limit().await;
    }
}
