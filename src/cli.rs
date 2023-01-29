use crate::things::{Thing, ThingType};
use clap::Parser;
use futures_util::pin_mut;
use futures_util::stream::StreamExt;
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;

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

    /// What "things" you want to delete (e.g. `comments`, `posts`).
    #[clap(long, env = "SHREDDIT_THINGS", default_values = &["posts", "comments"], value_delimiter = ',')]
    pub things: Vec<ThingType>,
}

impl Config {
    pub async fn run(&self, client: &Client, access_token: &str) {
        for thing_type in &self.things {
            let things = Thing::list(client, thing_type, &self.username).await;

            pin_mut!(things);

            while let Some(thing) = things.next().await {
                sleep(Duration::from_secs(2)).await; // Reddit has a rate limit

                thing
                    .edit(&client, &access_token, self.dry_run)
                    .await
                    .unwrap();

                sleep(Duration::from_secs(2)).await; // Reddit has a rate limit

                thing.delete(client, access_token, self.dry_run).await;
            }
        }
    }
}
