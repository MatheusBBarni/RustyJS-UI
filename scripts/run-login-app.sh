#!/bin/bash

set -euo pipefail

PORT="3000"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
API_DIR="$REPO_ROOT/examples/login-app/rest-api"
APP_ENTRY="$REPO_ROOT/examples/login-app/app/main.js"
API_URL="http://127.0.0.1:${PORT}/"
STDOUT_LOG="$(mktemp -t rustyjs-login-app-api.XXXXXX.out.log)"
STDERR_LOG="$(mktemp -t rustyjs-login-app-api.XXXXXX.err.log)"
API_PID=""

cleanup() {
    local exit_code=$?

    if [[ -n "$API_PID" ]] && kill -0 "$API_PID" 2>/dev/null; then
        kill "$API_PID" 2>/dev/null || true
        wait "$API_PID" 2>/dev/null || true
    fi

    rm -f "$STDOUT_LOG" "$STDERR_LOG"
    exit "$exit_code"
}

require_command() {
    local name=$1
    if ! command -v "$name" >/dev/null 2>&1; then
        echo "Required command not found: $name" >&2
        exit 1
    fi
}

wait_for_api() {
    local attempt

    for attempt in $(seq 1 60); do
        if ! kill -0 "$API_PID" 2>/dev/null; then
            echo "Login app API exited before becoming ready." >&2
            echo "STDOUT:" >&2
            cat "$STDOUT_LOG" >&2 || true
            echo "STDERR:" >&2
            cat "$STDERR_LOG" >&2 || true
            exit 1
        fi

        if curl --silent --fail --max-time 1 "$API_URL" >/dev/null 2>&1; then
            return
        fi

        sleep 0.5
    done

    echo "Timed out waiting for login app API at $API_URL" >&2
    echo "STDOUT:" >&2
    cat "$STDOUT_LOG" >&2 || true
    echo "STDERR:" >&2
    cat "$STDERR_LOG" >&2 || true
    exit 1
}

trap cleanup EXIT INT TERM

require_command bun
require_command cargo
require_command curl

echo "Starting login app API on $API_URL"
(
    cd "$API_DIR"
    PORT="$PORT" bun run start >"$STDOUT_LOG" 2>"$STDERR_LOG"
) &
API_PID=$!

wait_for_api

echo "API ready. Launching RustyJS-UI app..."
cd "$REPO_ROOT"
cargo run -- "$APP_ENTRY"
