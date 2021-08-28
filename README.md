# dj_ferris

Discord music bot written in rust. Uses [serenity-rs](https://github.com/serenity-rs) and [Lavalink](https://github.com/freyacodes/Lavalink) with [lavalink-rs](https://gitlab.com/vicky5124/lavalink-rs/).

## Requirements

- [Rust](https://www.rust-lang.org/)

## Dependencies

You need to install following dependencies:

- Opus

apt

```bash
apt install libopus-dev
```

pacman

```bash
pacman -S opus
```

dnf

```bash
dnf install opus-devel
```

## Installation

### Plugins

Add youtube-plugin-{version}.jar to plugins/ folder. You can get it from here: https://github.com/lavalink-devs/youtube-source/releases. Make sure that the version you downloaded and the version in application.yml match.

### Config file

Copy application-example.yml and rename it to application.yml. This file is used to configure lavalink.

### Environment

DJ Ferris requires a few environment variables which can be set either with export command or .env file.
Required variables are:

- DISCORD_TOKEN
- APPLICATION_ID
- LAVALINK_SERVER_PASSWORD

### Development build

```bash
# You can also use .env file for these
export DISCORD_TOKEN={bot token} # Set bot token env variable
export APPLICATION_ID={application id} # Set application id
export LAVALINK_SERVER_PASSWORD={lavalink password} # From application.yml

docker-compose -f docker-compose-lavalink.yml up # Run Lavalink with docker
cargo run
```

## Release build with docker

```bash
# Build docker image
docker build . -t dj-ferris

# Run docker image
docker run -it --name "dj-ferris" -e DISCORD_TOKEN=<DISCORD BOT TOKEN> -e APPLICATION_ID=<DISCORD APPLICATION ID> dj-ferris

OR

# Run with docker compose
docker-compose up
```
