# Bobby Turtle Bot

a virtual turtle pet on telegram.

This bot is under **active development**.

## Usage

Use `/help` in chat.

## Local development

you'll need a reverse proxy, such as ngrok.

```sh
ngrok http 3000
```

copy paste the url into `.env` , key = `APP_APPLICATION__PUBLIC_URL`.

The reverse proxy URL changes everytime you use ngrok command, so you will need
to change the url everytime.
