#!/usr/bin/env bash
# shellcheck disable=SC1090,SC1091
set -e
set -u

ROOT_DIR=$(git rev-parse --show-toplevel)
. "$ROOT_DIR/tools/general.sh"

cd "$ROOT_DIR"

fmt_args=()
if ci_is_running; then
    fmt_args+=("--check")
fi

cargo --version
cargo fmt --version

print_info "Run Rust format."
cargo fmt "${fmt_args[@]}" "$@"

print_info "Done."
