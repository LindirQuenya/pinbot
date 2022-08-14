#!/usr/bin/env bash
docker run -d --rm -e "DISCORD_TOKEN=$(cat token.txt)" rust-pinbot:latest
