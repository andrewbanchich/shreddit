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
- I found that [Python Shreddit](https://github.com/x89/Shreddit) didn't delete comments which mods removed. This does.

# Installation

## Manual

Download the binary from the [GitHub Releases](https://github.com/andrewbanchich/shreddit/releases) page.

## Cargo

`cargo install --version 0.1.0-alpha.2 shreddit` (change `0.1.0-alpha.2` to the latest version)

# How to use

```
USAGE:
    shreddit --username <USERNAME> --password <PASSWORD> --client-id <CLIENT_ID> --client-secret <CLIENT_SECRET>

OPTIONS:
        --client-id <CLIENT_ID>            [env: SHREDDIT_CLIENT_ID=]
        --client-secret <CLIENT_SECRET>    [env: SHREDDIT_CLIENT_SECRET=]
    -h, --help                             Print help information
    -p, --password <PASSWORD>              [env: SHREDDIT_PASSWORD=]
    -u, --username <USERNAME>              [env: SHREDDIT_USERNAME=]
    -V, --version                          Print version information
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

## Other features

These are the other features [Python Shreddit had](https://github.com/x89/Shreddit/blob/master/shreddit.yml.example).

I'll be adding these as I go along. PRs are welcome!

- [x] Dry run - preview what would happen with given configuration.
- [ ] Hours of comments you want to preserve.
- [ ] Max score - preserve comments with a score higher than this.
- [ ] Comment sorting
- [ ] Clear vote - Remove your votes before deleting.
- [ ] Item - configure what kinds of items to delete (submissions, comments, etc.)
- [ ] Subreddit whitelist - anything in given subreddits will not be deleted.
- [ ] Whitelist IDs - preserve specific posts by listing their IDs.
- [ ] Preserve distinguished - Don't deleted distinguished comments.
- [ ] Preserve gilded - Don't deleted gilded comments.

Other feature ideas are welcome.
