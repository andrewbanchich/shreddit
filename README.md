<div align="center">
<img src="assets/shreddit.svg" width="30%">
<h1>shreddit</h1>
</div>

[![Crates.io](https://img.shields.io/crates/v/shreddit.svg)](https://crates.io/crates/shreddit)

`shreddit` is a tool to delete Reddit comments and posts.

Deactivating a Reddit account will not delete comments or submissions - it will only disassociate your account from them.

Shreddit overwrites your comments with random text before deleting them to ensure that the originals are (probably) not preserved.

If you don't want your post history to follow you around forever, you can use `shreddit` on a cron job.

If you're deactivating your account, you can run `shreddit` first to ensure your posts are deleted.

# About

Because the [original Shreddit](https://github.com/x89/Shreddit) project was abandoned in 2017,
I decided to rewrite it in Rust.

This brings several benefits:

- You don't need to have Python or anything else installed.
- Instead of having `shreddit.yml` and `praw.ini` config files, all configuration can be done through CLI commands
with environment variables as default fallbacks.

# Installation

## Manual

Download the binary from the [GitHub Releases](https://github.com/andrewbanchich/shreddit/releases) page.

## Cargo

`cargo install shreddit`

# How to use

## Create Reddit app credentials

1. Navigate to Reddit -> preferences -> apps (tab) and click `create another app...`
   - Access page directly at https://www.reddit.com/prefs/apps
2. Give the app a name, like 'shreddit`. The name doesn't matter.
3. Select `script`.
4. Set the redirect URL to be `http://localhost:8080`.
5. Click `create app`.

This will provide with a client ID and client secret. The `CLIENT_ID` value used by Shreddit is shown under the name of the app you created. The `CLIENT_SECRET` is shown after clicking `edit`.

> IMPORTANT: If you are using TOTP, you will need to pass it in via the `PASSWORD` settings like `PASSWORD:TOTP`. It's currently a bit awkward to do since you need to do so before the token expires, so in the future we can add a CLI prompt to make this easier. In the meantime, you can just use the `--password` CLI argument, fill in `--password your-password:` and paste the TOTP after that and hit enter.

```
Overwrite and delete your Reddit account history.

Usage: shreddit [OPTIONS] --username <USERNAME> --password <PASSWORD> --client-id <CLIENT_ID> --client-secret <CLIENT_SECRET>

Options:
  -u, --username <USERNAME>
          Your Reddit username [env: SHREDDIT_USERNAME=]
  -p, --password <PASSWORD>
          Your Reddit password [env: SHREDDIT_PASSWORD=]
      --client-id <CLIENT_ID>
          To create client credentials, you need to navigate to `https://www.reddit.com/prefs/apps/`, click `create another app...` and fill out the form. Select the `script` type, and set `redirect uri` as `http://localhost:8080` [env: SHREDDIT_CLIENT_ID=]
      --client-secret <CLIENT_SECRET>
          The client secret from when you created client credentials [env: SHREDDIT_CLIENT_SECRET=]
      --dry-run
          If set, shreddit will not modify or delete anything. It will simply log what it would do if not in dry run mode. This allows you to preview the plan of action before executing [env: SHREDDIT_DRY_RUN=]
      --thing-types <THING_TYPES>
          What "things" you want to delete [env: SHREDDIT_THING_TYPES=] [default: posts comments] [possible values: posts, comments, friends, saved-posts, saved-comments]
      --before <BEFORE>
          Don't alter things that were created before this date.
          [env: SHREDDIT_BEFORE=] [default: "2023-06-25 14:42:43.828192320 UTC"]
      --after <AFTER>
          Don't alter things that were created after this date.
          [env: SHREDDIT_AFTER=] [
      --max-score <MAX_SCORE>
          [env: SHREDDIT_MAX_SCORE=]
  -r, --replacement-comment <REPLACEMENT_COMMENT>
          Allows a user to specify a custom string as their comment replacement text [env: SHREDDIT_REPLACEMENT_COMMENT=] [default: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."]
      --user-agent <USER_AGENT>
          The User-Agent for Reddit API requests [env: SHREDDIT_USER_AGENT=] [default: ShredditRustClient]
      --gdpr-export-dir <GDPR_EXPORT_DIR>
          The path of the directory of the unzipped GDPR export data. If set, `shreddit` will use the GDPR export folder instead of Reddit's APIs for discovering your data [env: SHREDDIT_GDPR_EXPORT_DIR=]
      --edit-only
          If specified, comments will only be edited, not deleted. - Requires gdpr_export [env: SHREDDIT_EDIT_ONLY=]
  -h, --help
          Print help
  -V, --version
          Print version
```

You can choose to pass in configuration settings via CLI arguments like:

```
shreddit --username YouRedditUsername --password YourSuperSecretPassword123  --client-id k1jh2342k3j --client-secret 2345JHLJ_34kjhkj3h453453
```

or by setting them as environment variables (e.g. `SHREDDIT_CLIENT_SECRET`) and simply running `shreddit`.

On startup, `shreddit` looks for a `shreddit.env` file in the current directory and sets any variables declared there.
However, this is purely optional.

## Dry run

You can use `--dry-run` or `SHREDDIT_DRY_RUN=true` to see what it would do without it actually doing anything.

## Delete ALL your data using GDPR export

1. Request an archive of all your data by [following these steps](https://reddit.zendesk.com/hc/en-us/articles/360043048352-How-do-I-request-a-copy-of-my-Reddit-data-and-information-).
2. Download the archive and extract it.
3. Run `shreddit` with the `--gdpr-export-dir` flag set to the path of the directory it was extracted to.

## Other features

These are the other features [Python Shreddit had](https://github.com/x89/Shreddit/blob/master/shreddit.yml.example).

I'll be adding these as I go along. PRs are welcome!

- [x] Dry run - preview what would happen with given configuration.
- [x] Preserve comments made after a given datetime.
- [x] Max score - preserve comments with a score higher than this.
- [ ] Comment sorting
- [ ] Clear vote - Remove your votes before deleting.
- [x] Item - configure what kinds of items to delete (submissions, comments, etc.)
- [ ] Subreddit whitelist - anything in given subreddits will not be deleted.
- [ ] Whitelist IDs - preserve specific posts by listing their IDs.
- [ ] Preserve distinguished - Don't deleted distinguished comments.
- [ ] Preserve gilded - Don't deleted gilded comments.

Other feature ideas are welcome.
