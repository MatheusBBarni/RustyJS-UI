#!/bin/bash

# Default values
EXAMPLE="hello_world_counter.js"
PRINT_ONLY=false

# Simple argument parsing
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --print-only) PRINT_ONLY=true ;;
        *) EXAMPLE="$1" ;;
    esac
    shift
done

# Get repository root (parent of the scripts directory)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

# Check if the path is absolute
# For Windows-style absolute paths (C:\) or Linux-style (/)
if [[ "$EXAMPLE" =~ ^/ ]] || [[ "$EXAMPLE" =~ ^[A-Za-z]: ]] || [[ "$EXAMPLE" =~ ^// ]]; then
    EXAMPLE_PATH="$EXAMPLE"
elif [[ -f "$REPO_ROOT/$EXAMPLE" ]]; then
    EXAMPLE_PATH="$REPO_ROOT/$EXAMPLE"
else
    EXAMPLE_PATH="$REPO_ROOT/examples/$EXAMPLE"
fi

if [[ ! -f "$EXAMPLE_PATH" ]]; then
    echo "Example not found: $EXAMPLE_PATH"
    exit 1
fi

# Switch to repo root
cd "$REPO_ROOT" || exit 1

if [ "$PRINT_ONLY" = true ]; then
    echo "cargo run -- \"$EXAMPLE_PATH\""
    exit 0
fi

cargo run -- "$EXAMPLE_PATH"
