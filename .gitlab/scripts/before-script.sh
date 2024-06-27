#!/usr/bin/env bash
# shellcheck disable=SC1090,SC1091
# This script is sourced.
set -u

ROOT_DIR=$(git rev-parse --show-toplevel)
. "$ROOT_DIR/tools/general.sh"

# ci_container_mgr_setup

unset ROOT_DIR
