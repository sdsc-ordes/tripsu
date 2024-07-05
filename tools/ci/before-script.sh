#!/usr/bin/env bash
# shellcheck disable=SC1090,SC1091
# This script is sourced in each step.
set -u

git config --global safe.directory "*" || {
    echo "Could not overwrite safe.directory in Git config." >&2
    exit 1
}

ROOT_DIR=$(git rev-parse --show-toplevel)
. "$ROOT_DIR/tools/general.sh"

print_info "Current dir: '$(pwd)'"
print_info "Running as user: $(id)"

ci_setup_git

unset ROOT_DIR
