use crate::things::{Thing, ThingType};
use chrono::{DateTime, Utc};
use clap::Parser;
use futures_util::pin_mut;
use futures_util::stream::StreamExt;
use reqwest::Client;
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
            info!("Shredding {thing_type:?}");

            // TODO For some reason, pagination doesn't continue for all Things
            // without this outer loop. Find out why Thing::list() doesn't continue to the end
            // of all Things.
            loop {
                let things = Thing::list(client, thing_type, &self.username).await;

                pin_mut!(things);

                if let Some(thing) = things.next().await {
                    thing.shred(self, client, access_token).await;
                } else {
                    debug!("Completed listing {thing_type:?}");
                    break;
                }

                while let Some(thing) = things.next().await {
                    thing.shred(self, client, access_token).await;
                }
            }
        }
    }
}
