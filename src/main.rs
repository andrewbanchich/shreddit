use std::error::Error;

use access_token::new_access_token;
use clap::Parser;
use cli::Config;
use reqwest::Client;
use tracing::error;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod access_token;
mod cli;
mod things;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::from_filename("shreddit.env").ok();

    let config = Config::parse();

    init_tracing();

    let client = Client::new();
    let access_token = match new_access_token(&config, &client).await {
        Ok(token) => token,
        Err(e) => {
            error!("{e}");
            return Err(e.into());
        }
    };

    config.run(&client, &access_token).await;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("shreddit"))
        .unwrap();

    let format = fmt::layer().with_target(false).pretty();

    tracing_subscriber::registry()
        .with(filter)
        .with(format)
        .init();
}
