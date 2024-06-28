#!/usr/bin/env bash
# shellcheck disable=SC1090,SC1091
set -e
set -u

ROOT_DIR=$(git rev-parse --show-toplevel)
. "$ROOT_DIR/tools/general.sh"

cd "$ROOT_DIR"

print_info "Run Rust format."

fmt_args=()
if ci_is_running; then
    fmt_args+=("--check")
fi

ci_wrap_container \
    ghcr.io/sdsc-ordes/rdf-protect:ci-format-1.0.0 \
    nix develop ./tools/nix#ci --command \
    cargo fmt "${fmt_args[@]}" "$@"
