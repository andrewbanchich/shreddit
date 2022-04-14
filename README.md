```
███████╗██╗  ██╗██████╗ ███████╗██████╗ ██████╗ ██╗████████╗
██╔════╝██║  ██║██╔══██╗██╔════╝██╔══██╗██╔══██╗██║╚══██╔══╝
███████╗███████║██████╔╝█████╗  ██║  ██║██║  ██║██║   ██║   
╚════██║██╔══██║██╔══██╗██╔══╝  ██║  ██║██║  ██║██║   ██║   
███████║██║  ██║██║  ██║███████╗██████╔╝██████╔╝██║   ██║   
╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚═════╝ ╚═════╝ ╚═╝   ╚═╝   
```

Because the [Shreddit](https://github.com/x89/Shreddit) project was abandoned in 2017,
I decided to rewrite it in Rust.

This brings several benefits:

- You don't need to have Python or anything else installed.
- You don't need to worry about what version of Python is installed.
- Instead of having `shreddit.yml` and `praw.ini` config files, all configuration is done through CLI commands
with environment variables as default fallbacks.
- It isn't Python.

# Installation

## Manual

Download the binary from the [GitHub Releases](https://github.com/andrewbanchich/shreddit/releases) page.

## Cargo

`cargo install shreddit`

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
