use std::time::Duration;

use crate::things::{Thing, ThingType};
use chrono::{DateTime, Utc};
use clap::Parser;
use futures_util::pin_mut;
use futures_util::stream::StreamExt;
use reqwest::Client;
use tokio::time::sleep;
use tracing::{debug, info};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Config {
    /// Your Reddit username.
    #[clap(short, long, env = "SHREDDIT_USERNAME")]
    pub username: String,

    /// Your Reddit password.
    #[clap(short, long, env = "SHREDDIT_PASSWORD")]
    pub password: String,

    /// To create client credentials, you need to navigate to `https://www.reddit.com/prefs/apps/`,
    /// click `create another app...` and fill out the form. Select the `script` type,
    /// and set `redirect uri` as `http://localhost:8080`.
    #[clap(long, env = "SHREDDIT_CLIENT_ID")]
    pub client_id: String,

    /// The client secret from when you created client credentials.
    #[clap(long, env = "SHREDDIT_CLIENT_SECRET")]
    pub client_secret: String,

    /// If set, shreddit will not modify or delete anything. It will simply log what it would do
    /// if not in dry run mode. This allows you to preview the plan of action before executing.
    #[clap(long, env = "SHREDDIT_DRY_RUN")]
    pub dry_run: bool,

    /// What "things" you want to delete.
    #[clap(long, env = "SHREDDIT_THINGS", default_values = &["posts", "comments"], value_delimiter = ',')]
    pub things: Vec<ThingType>,

    #[clap(long, env = "SHREDDIT_BEFORE", default_value_t = Utc::now())]
    pub before: DateTime<Utc>,

    #[clap(long, env = "SHREDDIT_MAX_SCORE")]
    pub max_score: Option<i64>,
}

impl Config {
    pub async fn run(&self, client: &Client, access_token: &str) {
        for thing_type in &self.things {
            info!("Shredding {thing_type:?}...");

            let things = Thing::list(client, thing_type, &self.username).await;

            pin_mut!(things);

            while let Some(thing) = things.next().await {
                if should_skip(&thing, self) {
                    continue;
                } else {
                    thing.shred(self, client, access_token).await;
                    sleep(Duration::from_secs(2)).await; // Reddit has a rate limit
                }
            }

            debug!("Completed listing {thing_type:?}");
        }
    }
}

fn should_skip(thing: &Thing, config: &Config) -> bool {
    if thing.created() >= config.before {
        debug!("Skipping due to `before` filter ({})", config.before);
        return true;
    } else if let Some(max_score) = config.max_score {
        if thing.score() > max_score {
            debug!("Skipping due to `max_score` filter ({max_score})");

            return true;
        }
    }

    false
}
