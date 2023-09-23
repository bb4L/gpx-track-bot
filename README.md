# gpx-track-bot

[deployed bot](https://t.me/GpxTrackBot)


## deploy with docker

```
services:
  bot:
    image: ghcr.io/bb4l/gpx-track-bot:latest
    working_dir: /home/gxp_bot
    environment:
      TELOXIDE_TOKEN: YOUR_BOT_TOKEN
      GPX_TRACK_BOT_ALLOWED_USERS: ALLOWED_USERS
    command: gpx-bot
    volumes:
      - ./path/to/store/gpx/files:/gpx_files
```