#!/usr/bin/env bash
# shellcheck disable=SC1090,SC1091
set -e
set -u

ROOT_DIR=$(git rev-parse --show-toplevel)
. "$ROOT_DIR/tools/general.sh"

cd "$ROOT_DIR"

cargo --version
cargo clippy --version

print_info "Run Rust Clippy linter."
print_warning "Currently warnings are not errors!"

cargo clippy --no-deps -- -A clippy::needless_return "$@" ||
    {
        git diff --name-status || true
        die "Rust clippy failed."
    }

print_info "Done."
