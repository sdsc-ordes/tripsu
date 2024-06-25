#!/usr/bin/env bash
# shellcheck disable=SC1090,SC1091
set -e
set -u

ROOT_DIR=$(git rev-parse --show-toplevel)
. "$ROOT_DIR/tools/general.sh"

cd "$ROOT_DIR"

print_info "Run Rust Clippy linter."
ci_wrap_container \
    ghcr.io/sdsc-ordes/rdf-protect:ci-lint-1.0.0 \
    nix develop ./tools/nix#ci --command \
    cargo clippy --no-deps -- -A clippy::needless_return "$@" ||
    die "Rust clippy failed."

print_info "Run Rust Miri to check undefined behaviour."
ci_wrap_container \
    ghcr.io/sdsc-ordes/rdf-protect:ci-lint-1.0.0 \
    nix develop ./tools/nix#ci --command \
    cargo miri test "$@" ||
    die "Rust Miri failed."
