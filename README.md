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
