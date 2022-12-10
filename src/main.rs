use clap::Parser;
use futures_util::pin_mut;
use futures_util::stream::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use std::{collections::HashMap, time::Duration};
use things::{Thing, ThingType};
use tokio::time::sleep;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod things;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Shreddit {
    #[clap(short, long, env = "SHREDDIT_USERNAME")]
    username: String,
    #[clap(short, long, env = "SHREDDIT_PASSWORD")]
    password: String,
    #[clap(long, env = "SHREDDIT_CLIENT_ID")]
    client_id: String,
    #[clap(long, env = "SHREDDIT_CLIENT_SECRET")]
    client_secret: String,
    #[clap(long, env = "SHREDDIT_DRY_RUN")]
    dry_run: bool,
    #[clap(long, env = "SHREDDIT_THINGS", default_values = &["posts", "comments"])]
    things: Vec<ThingType>,
}

impl Shreddit {
    async fn run(&self, client: &Client, access_token: &str) {
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
                // match &thing {
                //     Thing::Post { title, .. } => {
                //         info!("Deleting post: `{title}`");

                //         thing.delete(client, access_token, self.dry_run).await;
                //     }
                //     Thing::Comment { id, .. } => {
                //     }
                // }
            }
        }
    }

    async fn new_access_token(&self, client: &Client) -> String {
        let mut params = HashMap::new();
        params.insert("grant_type", "password");
        params.insert("username", &self.username);
        params.insert("password", &self.password);

        let res: AccessTokenResponse = client
            .post("https://www.reddit.com/api/v1/access_token")
            .form(&params)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        res.access_token
    }
}

#[tokio::main]
async fn main() {
    dotenv::from_filename("shreddit.env").ok();

    // Parse CLI
    let shreddit = Shreddit::parse();

    // Initialize tracing
    {
        let filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("shreddit"))
            .unwrap();

        let format = fmt::layer().with_target(false);

        tracing_subscriber::registry()
            .with(filter)
            .with(format)
            .init();
    }

    let client = Client::new();
    let access_token = shreddit.new_access_token(&client).await;

    shreddit.run(&client, &access_token).await;

    // if ARGS.things.contains(&ThingType::Comments) {
    //     loop {
    //         // Delete comments
    //         {
    //             let comms = list_things(ThingType::Comments).await;

    //             if comms.is_empty() {
    //                 break;
    //             }

    //             let comment_count = comms.len();
    //             let possible_s = if comment_count == 1 { "" } else { "s" };

    //             info!("Deleting {} comment{}...", comment_count, possible_s);

    //             for comment in comms {
    //                 let fullname = format!("t1_{}", comment.data.id);
    //                 // CommentObj::edit(&res.access_token, &format!("t1_{}", comment.data.id)).await;
    //                 delete_thing(&fullname, &res.access_token).await;
    //             }
    //         }
    //     }
    // }
}

#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    // expires_in: String,
    // scope: String,
    // token_type: String,
}

// async fn me(access_token: &str) {
//     let mut headers = HeaderMap::new();
//     headers.insert(
//         "Authorization",
//         format!("Bearer {access_token}").parse().unwrap(),
//     );

//     headers.insert("User-Agent", format!("ShredditClient/0.1").parse().unwrap());

//     let res: Value = REQWEST
//         .get("https://oauth.reddit.com/api/v1/me")
//         .headers(headers)
//         .send()
//         .await
//         .unwrap()
//         .json()
//         .await
//         .unwrap();

//     dbg!(res);
// }
