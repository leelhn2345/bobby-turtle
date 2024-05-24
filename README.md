# Telegram Bot

<!--toc:start-->
- [Telegram Bot](#telegram-bot)
  - [Features](#features)
  - [Usage](#usage)
  - [Development](#development)
    - [Requirements](#requirements)
    - [Setup](#setup)
  - [Production](#production)
    - [Build](#build)
    - [Migration](#migration)
    - [Prod Env](#prod-env)
<!--toc:end-->

A virtual [turtle pet](https://t.me/baldyturtlebot) on telegram.

## Features

- reminder

<https://github.com/leelhn2345/turtle-bot/assets/64735315/f5c66eee-9e05-46b6-9f18-5010eab3d7de>

- chatgpt integration

<https://github.com/leelhn2345/turtle-bot/assets/64735315/ca3b868a-e9cf-459d-bac2-3e69e0e517c3>

## Usage

Use `/help` in chat.

When **it's not a group chat**, such as direct messages to the bot, the bot will
reply to every message.

**In group chats**, use `/chat` for it to reply to every message. Use `/shutup`
for the bot to stop responding.

## Development

Go to `http://<address>/<port>/docs` for app's Swagger UI.

The default address and port has been configured in [`base.yaml`](./config/base.yaml)
and [`local.yaml`](./config/local.yaml).

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
ngrok --http domain=xxxx.xxx.xxx.app 8443
# only available ports for webhooks are 80, 88, 443 or 8443
```

Copy [`.env.template`](./.env.template) to `.env` and fill up the corresponding
environment values.

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

## Production

### Build

to build app without communicating with database:

```sh
cargo sqlx prepare
# or run `make prep`
```

### Migration

To migrate production database:

```sh
DATABASE_URL=<connection_string> sqlx migrate run
```

### Prod Env

Default configs are in [`./config`](./config) directory.

For **runtime** or **sensitive** environment values, please refer to [`.env.prod`](./.env.prod).
