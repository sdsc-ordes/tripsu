#!/usr/bin/env bash
# shellcheck disable=SC1090,SC1091
#
# Assert that the release tag exists
# and check that its on main.
# On `--push` do push the tag.

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
. "$ROOT_DIR/tools/general.sh"

cd "$ROOT_DIR"

RELEASE_BRANCH="main"

function main() {
    local type="$1"
    local prepare_tag="$2"

    local release_tag=${prepare_tag##*prepare-}

    if [ "$type" = "push" ]; then
        print_info "Pushing tag '$release_tag'."

        git push origin "$release_tag" ||
            die "Could not push tag."

    elif [ "$type" = "create-and-check" ]; then

        print_info "Create tag '$release_tag' and check."

        # Gets the message on the annotated commit:
        deref() {
            git for-each-ref "refs/tags/$release_tag" --format="%($1)"
        }

        deref contents

        # Creates a new tag with the same message,
        # including trailing headers.
        git tag -a -m "$(deref contents)" "$release_tag" ||
            die "Could not create tag."

        # Fetch the branch.
        git fetch --depth 50 origin "$RELEASE_BRANCH"

        # Check if its reachable.
        if [ -n "$(git rev-list --first-parent \
            --ancestry-path \
            "$release_tag^..origin/$RELEASE_BRANCH")" ]; then
            die "Tag is not reachable from '$RELEASE_BRANCH' (--first-parent) !"
        fi

    elif [ "$type" = "cleanup" ]; then

        print_info "Cleanup the prepare tag."
        git tag -d "$prepare_tag" || true
        git push origin :"$prepare_tag" || true

    else
        die "Type '$type' is not known."
    fi
}

main "$@"
