#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 [target-triple]"
  echo "Packages a release build into dist/."
  echo "Example: $0 x86_64-unknown-linux-gnu"
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

target="${1:-}"
package_name="oasis-weather-notify"
version=$(rg '^version\s*=' Cargo.toml | head -n1 | cut -d '"' -f2)

if [[ -n "$target" ]]; then
  cargo build --locked --release --target "$target"
  target_dir="target/$target/release"
  archive_target="$target"
else
  cargo build --locked --release
  target_dir="target/release"
  archive_target=$(rustc -vV | rg '^host:' | awk '{print $2}')
fi

binary_path="$target_dir/$package_name"
if [[ ! -f "$binary_path" ]]; then
  echo "Binary not found at $binary_path" >&2
  exit 1
fi

mkdir -p dist
archive_name="$package_name-$version-$archive_target.tar.gz"

tar -C "$target_dir" -czf "dist/$archive_name" "$package_name"

echo "Created dist/$archive_name"
