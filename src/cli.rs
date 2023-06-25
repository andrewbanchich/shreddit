use crate::things::{ThingType, LOREM_IPSUM};
use chrono::{DateTime, Utc};
use clap::Parser;
use std::path::PathBuf;

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
    #[clap(long, env = "SHREDDIT_THING_TYPES", default_values = &["posts", "comments"], value_delimiter = ',')]
    pub thing_types: Vec<ThingType>,

    #[clap(long, env = "SHREDDIT_BEFORE", default_value_t = Utc::now())]
    pub before: DateTime<Utc>,

    #[clap(long, env = "SHREDDIT_MAX_SCORE")]
    pub max_score: Option<i64>,

    /// Allows a user to specify a custom string as their comment replacement text
    #[clap(short, long, env = "SHREDDIT_REPLACEMENT_COMMENT", default_value = LOREM_IPSUM)]
    pub replacement_comment: String,

    /// The User-Agent for Reddit API requests.
    #[clap(
        long,
        env = "SHREDDIT_USER_AGENT",
        default_value = "ShredditRustClient"
    )]
    pub user_agent: String,

    /// The path of the directory of the unzipped GDPR export data.
    /// If set, `shreddit` will use the GDPR export folder instead of
    /// Reddit's APIs for discovering your data.
    #[clap(long, env = "SHREDDIT_GDPR_EXPORT_DIR")]
    pub gdpr_export_dir: Option<PathBuf>,
}
