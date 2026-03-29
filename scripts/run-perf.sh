#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$REPO_ROOT"

if [[ "${1:-}" == "--release" ]]; then
  cargo run --bin perf_harness --release
else
  cargo run --bin perf_harness
fi
