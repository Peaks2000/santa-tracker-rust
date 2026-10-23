#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
task_needs_build=0
for task_source in Cargo.toml Cargo.lock src/*.rs data/* assets/fonts/*.ttf; do
  if [[ "$task_source" -nt target/release/santa-tracker ]]; then
    task_needs_build=1
  fi
done
if [[ -x target/release/santa-tracker && "$task_needs_build" == 0 ]]; then
  exec target/release/santa-tracker "$@"
fi
if ! command -v "${CARGO:-cargo}" >/dev/null 2>&1; then
  echo 'Install Rust (https://rustup.rs), then run: cargo build --release --manifest-path rust/Cargo.toml' >&2
  exit 1
fi
./build.sh
exec target/release/santa-tracker "$@"
