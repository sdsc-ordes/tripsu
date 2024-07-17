#!/usr/bin/env bash
# shellcheck disable=SC1090,SC1091
# This script is sourced in each step.
set -u
set -e

git config --global safe.directory "*" || {
    echo "Could not overwrite safe.directory in Git config." >&2
    exit 1
}

ROOT_DIR=$(git rev-parse --show-toplevel)
. "$ROOT_DIR/tools/general.sh"

# Some home workaround for this issue:
# https://github.com/actions/runner/issues/863
# Why, really whyyyy is Github overwriting the HOME directory!
if [ "$HOME" = "/github/home" ]; then
    print_warning "Making symlink for '$HOME' to /root to" \
        "workaround some Github stupidity."

    ls -al "/github" || true
    ls -al "/github/home" || true

    rm -rf /github/home || true
    ln -s /root "$HOME"
    ls -al "/github"
fi

ci_setup_git
