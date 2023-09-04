# Bobby Turtle Bot

a virtual turtle pet on telegram.
[`[@bobbyturtlebot](https://t.me/bobbyturtlebot)

This bot is **still in active development**.

## Usage

Use `/help` in chat.

## Local development

### requisite

Webhooks are used in this bot. So we need a reverse proxy.

Use either [`ngrok`](https://ngrok.com/) or
[`localtunnel`](https://github.com/localtunnel/localtunnel) to set the
`base_url` in `config/local.yaml`

the default port is 3000.

```sh
# if use ngrok
ngrok http 3000
```

```sh
# if use localtunnel
# this is what i use
lt --port 3000 --subdomain bobby
```
