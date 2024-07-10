#!/usr/bin/env bash
# shellcheck disable=SC1090,SC1091
# This script is sourced in each step.
set -u
set -e

ROOT_DIR=$(git rev-parse --show-toplevel)
. "$ROOT_DIR/tools/general.sh"

function main() {
    local prepare_tag="$1"
    local repo="$2"

    local tag=${prepare_tag#prepare-}
    local version=${tag#v}

    print_info "Creating Github release ... "

    gh release create "$tag" \
        --repo="$repo" \
        --title="rdf-protect: $version" \
        --generate-notes

    print_info "Successfully created release. All done."
}

main "$@"
