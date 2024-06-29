#!/usr/bin/env bash
# shellcheck disable=SC1090,SC1091
#
# Creating a prepare tag to trigger the release process on the
# Github workflow. Can only be called on `main`.
#
# Usage: release.sh "1.2.0"

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
. "$ROOT_DIR/tools/general.sh"

function delete_prepare_tags() {
    readarray -t prepareTag < <(git tag --list "prepare-*")

    for tag in "${prepareTag[@]}"; do
        print_info "Deleting prepare tag '$tag'."
        git push -f origin ":${tag}" || true
        git tag -d "$tag"
    done
}

function commit_version_file() {
    local version="$1"
    print_info "Writing new version file... (for Nix)"

    temp=$(mktemp)
    jq ".version |= \"$version\"" "$VERSION_FILE" >"$temp"
    mv "$temp" "$VERSION_FILE"

    if ! git diff --quiet --exit-code; then
        git add "$VERSION_FILE"
        git commit -m "chore: update Nix package version to '$version'"
    fi
}

function create_tag() {
    tag="v$version"
    if git tag --list "v*" | grep -qE "^$tag$"; then
        print_info "Git tag '$tag' already exists."
        exit 1
    fi

    if git ls-remote "refs/tags/v*" | grep -qE "^$tag$"; then
        print_info "Git tag '$tag' already exists."
        exit 1
    fi

    print_info "Tagging..."
    git tag -a -m "Version $tag" "prepare-$tag"

    print_info "Tag contains:"
    git cat-file -p "prepare-$tag"
}

function trigger_build() {
    printf "Do you want to trigger the build? [y|n]: "
    read -r answer
    if [ "$answer" != "y" ]; then
        die "Do not trigger build -> abort."
    fi

    print_info "Pushing tag 'prepare-$tag'."
    git push -f origin --no-follow-tags "$branch" "prepare-$tag"
}

function main() {
    cd "$ROOT_DIR"

    version="$1"
    branch=$(git branch --show-current)

    if [ "$branch" != "main" ]; then
        die "Can only tag on 'main'."
    fi

    if ! git diff --quiet --exit-code; then
        die "You have changes on this branch."
    fi

    delete_prepare_tags
    commit_version_file "$version"
    create_tag
    trigger_build
}

main "$@"
