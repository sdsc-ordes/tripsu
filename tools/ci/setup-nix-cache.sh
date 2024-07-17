#!/usr/bin/env bash
# shellcheck disable=SC1090,SC1091
# This script is sourced in each step.
set -u
set -e

ROOT_DIR=$(git rev-parse --show-toplevel)
. "$ROOT_DIR/tools/general.sh"

ci_setup_cachix "${CACHIX_CACHE_NAME}" "${CACHIX_AUTH_TOKEN}"
