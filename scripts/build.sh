#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 [target-triple]"
  echo "Example: $0 x86_64-unknown-linux-gnu"
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

target="${1:-}"

if [[ -n "$target" ]]; then
  cargo build --locked --release --target "$target"
else
  cargo build --locked --release
fi
