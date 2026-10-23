#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
task_cargo="${CARGO:-cargo}"
if ! command -v "$task_cargo" >/dev/null 2>&1; then
  echo 'Rust is required to build this application. Install it from https://rustup.rs.' >&2
  exit 1
fi
# Some Linux systems ship libclang and its headers without the clang driver.
# Bindgen then cannot discover Clang's builtin header directory by itself.
if [[ -z "${BINDGEN_EXTRA_CLANG_ARGS:-}" ]] && ! command -v clang >/dev/null 2>&1; then
  for task_headers in /usr/lib/clang/*/include /usr/lib64/clang/*/include; do
    if [[ -f "$task_headers/stdbool.h" ]]; then
      export BINDGEN_EXTRA_CLANG_ARGS="-I$task_headers"
    fi
  done
fi
"$task_cargo" build --release "$@"
