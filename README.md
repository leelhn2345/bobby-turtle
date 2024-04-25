# Telegram Bot

<!--toc:start-->
- [Telegram Bot](#telegram-bot)
  - [Features](#features)
  - [Usage](#usage)
  - [Local Development](#local-development)
    - [Requirements](#requirements)
    - [Setup](#setup)
    - [Build](#build)
  - [Production](#production)
<!--toc:end-->

A virtual turtle pet on telegram.

> [!WARNING]
> This bot is under **active development**.

## Features

- cron-job
- chatgpt integration

## Usage

Use `/help` in chat.

## Local Development

Go to `http://<address>/<port>/docs` for app's Swagger UI.

### Requirements

- Reverse Proxy
  - Needed for webhook
  - I chose [ngrok](https://ngrok.com/) because there's no need to
  setup TLS for HTTPS.
  - Feel free to use any other reverse proxy.
- Postgres Db
- sqlx-cli
  - to run sql migrations
  - `cargo install sqlx-cli --no-default-features --features rustls postgres`
- Docker (Optional)

### Setup

This outline the steps I took for local development.

Run the reverse proxy and get the public URL.
In my case:

```sh
ngrok --http domain=xxxx.xxx.xxx.app 5000
```

Copy `.env.template` to `.env` and fill up the corresponding environment values.
`APP_APPLICATION__PUBLIC_URL` is the **public URL** for reverse proxy.

To start DB and trigger sql migrations:

```sh
make dev
```

If there is an error, likely it is caused by migration before database has been
fully setup. Just run the command again.

Run the app with:

```sh
cargo run
```

### Build

to build app without communicating with database:

```sh
cargo sqlx prepare
# or run `make prep`
```

## Production

To migrate production database:

```sh
DATABASE_URL=<connection_string> sqlx migrate run
```
