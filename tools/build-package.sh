#!/usr/bin/env bash
# shellcheck disable=SC1090,SC1091
#
# Build the Nix container image.

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
. "$ROOT_DIR/tools/general.sh"

cd "$ROOT_DIR"

dir="build/package"

print_info "Building the package."
nix --version
nix build -L "./tools/nix#tripsu" \
    --out-link "$dir" \
    "$@"
