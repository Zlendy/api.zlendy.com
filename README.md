# api.zlendy.com

This is the Rust codebase that powers [api.zlendy.com](https://api.zlendy.com/).
This project is meant to be used inside [zlendy.com](https://zlendy.com).

## Getting started

Copy `example.env` to `.env` and modify some values.

- **RUST_LOG**: Enables [logging](https://docs.rs/env_logger/latest/env_logger/#enabling-logging) to stdout.
- **UMAMI_URL** (MODIFY ME): URL of self-hosted [Umami Analytics](https://umami.is/docs/install) instance (does not work with Umami Analytics Cloud).
- **UMAMI_USERNAME** (MODIFY ME): Username used to access Umami Analytics API (please use a view-only user)
- **UMAMI_PASSWORD** (MODIFY ME): Password of aforementioned user.
- **UMAMI_WEBSITE_ID** (MODIFY ME): Can be found in "Settings > Websites > Website ID".
- **FEDIVERSE_URL**: URL of [Sharkey](https://activitypub.software/TransFem-org/Sharkey) instance. May work with [Misskey](https://misskey-hub.net) or its forks (untested)
- **FEDIVERSE_USER_ID**: Enable developer mode in "Settings > Other Settings > Other > Developer". Then go to the user profile and click on "... > Copy user ID".
- **ZLENDY_URL**: [zlendy.com](https://github.com/Zlendy/zlendy.com) instance. May be modified when testing new features in localhost:5173.

## Developing

(Optional) Install [cargo-watch](https://crates.io/crates/cargo-watch) to recompile the project on save.

```bash
# choose one
cargo run
cargo watch -x run
```

## Building

To create a production version of the project:

```bash
cargo build --release
```

## Deploying

This project is meant to be deployed using Docker.

```
docker compose up --build
```
