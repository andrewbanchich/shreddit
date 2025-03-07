use crate::things::{CommentIdSet, PostIdSet, SubredditSet, ThingType, LOREM_IPSUM};
use chrono::{DateTime, Utc};
use clap::Parser;
use std::path::PathBuf;
use parse_datetime::parse_datetime;
use tracing::{debug, warn};

/// Parses `SHREDDIT_BEFORE` to support:
/// - Absolute timestamps (ISO 8601) → `"2025-01-31T03:16:30Z"`
/// - Negative durations (`-30 days`, `-2 weeks`, `-5 hours`) → Converts to `Utc::now() - duration`
/// see https://github.com/uutils/parse_datetime for formats
fn parse_before(input: &str) -> Result<DateTime<Utc>, String> {
    // we only work with dates in the past
    if input.starts_with('-') {
        // Parse the duration part (excluding the '-')
        match parse_datetime(&input[1..]) {
            Ok(datetime) => {
                // Convert the parsed `DateTime<FixedOffset>` to `DateTime<Utc>`
                let datetime_utc = datetime.with_timezone(&Utc);
                // Calculate the duration by subtracting the current time
                let duration = datetime_utc - Utc::now();
                // Subtract the duration from the current time to get the target timestamp
                Ok(Utc::now() - duration)
            },
            Err(_) => Err(format!("Invalid duration format for SHREDDIT_BEFORE: {}", input)),
        }
    } else {
        // Try to parse as an absolute timestamp
        match parse_datetime(input) {
            Ok(datetime) => {
                // Convert the parsed `DateTime<FixedOffset>` to `DateTime<Utc>`
                let utc_timestamp = datetime.with_timezone(&Utc);
                Ok(utc_timestamp)
            },
            Err(_) => Err(format!("Invalid timestamp format for SHREDDIT_BEFORE: {}", input)),
        }
    }
}

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Config {
    /// Your Reddit username.
    #[clap(short, long, env = "SHREDDIT_USERNAME", allow_hyphen_values = true)]
    pub username: String,

    /// Your Reddit password.
    #[clap(short, long, env = "SHREDDIT_PASSWORD", allow_hyphen_values = true)]
    pub password: String,

    /// To create client credentials, you need to navigate to `https://www.reddit.com/prefs/apps/`,
    /// click `create another app...` and fill out the form. Select the `script` type,
    /// and set `redirect uri` as `http://localhost:8080`.
    #[clap(long, env = "SHREDDIT_CLIENT_ID", allow_hyphen_values = true)]
    pub client_id: String,

    /// The client secret from when you created client credentials.
    #[clap(long, env = "SHREDDIT_CLIENT_SECRET", allow_hyphen_values = true)]
    pub client_secret: String,

    /// If set, shreddit will not modify or delete anything. It will simply log what it would do
    /// if not in dry run mode. This allows you to preview the plan of action before executing.
    #[clap(long, env = "SHREDDIT_DRY_RUN")]
    pub dry_run: bool,

    /// What "things" you want to delete.
    #[clap(long, env = "SHREDDIT_THING_TYPES", default_values = &["posts", "comments"], value_delimiter = ',')]
    pub thing_types: Vec<ThingType>,

    /// Delete items before a specific date or duration (e.g., `-30d`).
    #[clap(long, env = "SHREDDIT_BEFORE", default_value_t = Utc::now(), value_parser = parse_before)]
    pub before: DateTime<Utc>,

    #[clap(long, env = "SHREDDIT_MAX_SCORE")]
    pub max_score: Option<i64>,

    /// Allows a user to specify a custom string as their comment replacement text
    #[clap(short, long, env = "SHREDDIT_REPLACEMENT_COMMENT", default_value = LOREM_IPSUM, allow_hyphen_values = true)]
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

    /// If specified, comments will only be edited, not deleted. - Requires gdpr_export
    #[clap(long, env = "SHREDDIT_EDIT_ONLY")]
    pub edit_only: bool,

    /// If specified, will skip these subreddits
    #[clap(long, env = "SHREDDIT_SKIP_SUBREDDITS")]
    pub skip_subreddits: Option<SubredditSet>,

    /// If specified, will skip comments and saved comments with listed ids
    #[clap(long, env = "SHREDDIT_SKIP_COMMENT_IDS")]
    pub skip_comment_ids: Option<CommentIdSet>,

    /// If specified, will skip posts and saved posts with listed ids
    #[clap(long, env = "SHREDDIT_SKIP_POST_IDS")]
    pub skip_post_ids: Option<PostIdSet>,
}

impl Config {
    /// Return TRUE if either edit_only or dr_run
    pub fn should_prevent_deletion(&self) -> bool {
        if self.edit_only {
            debug!(
                "Skipping DELETION due to `edit_only` filter ({})",
                self.edit_only
            );
            if self.gdpr_export_dir.is_none() {
                // As of this writing, there is an approx 1000 comment limit when pulling from JSON. Only reliable way to reach all data is via GDPR.
                // See issue #35: https://github.com/andrewbanchich/shreddit/issues/35
                warn!("Because you are not using a GDPR export, not all data will be reached.\nFor info on how to use a GDPR export, see: {}", r##"https://github.com/andrewbanchich/shreddit#delete-all-your-data-using-gdpr-export"##);
            }
        } else if self.dry_run {
            debug!("Skipping DELETION due to 'dry run' filter");
        }
        self.edit_only | self.dry_run
    }
}
