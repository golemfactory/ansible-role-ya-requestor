#!/bin/bash

die() {
    echo "$@"
    exit 1
}

. .env

ulimit -n 10000

if [[ -n "${YAGNA_API_URL+isset}" ]]; then
    die "use YAGNA_API_URL_IP and YAGNA_API_URL_PORT instead"
fi
: "${YAGNA_API_URL_IP:=127.0.0.1}"
: "${YAGNA_API_URL_PORT:=7465}"
export YAGNA_API_URL="http://${YAGNA_API_URL_IP}:${YAGNA_API_URL_PORT}"

exec yagna service run
