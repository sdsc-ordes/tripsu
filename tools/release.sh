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

RELEASE_BRANCH="${RELEASE_BRANCH:-main}"
VERSION_FILE="$ROOT_DIR/Cargo.toml"

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

    dasel put -r toml -f "$VERSION_FILE" -t string -v "$version" .package.version

    if ! git diff --exit-code --quiet; then
        # Commit if we have change.
        git add "$VERSION_FILE"
        git commit -m "chore: release '$version'"
    fi
}

function create_prepare_tag() {
    tag="v$version"

    print_info "Tagging..."
    git tag -a -m "Version $tag" "prepare-$tag"

    print_info "Tag contains:"
    git cat-file -p "prepare-$tag"
}

function trigger_build() {
    local branch="$1"
    printf "Do you want to trigger the build? [y|n]: "
    read -r answer
    if [ "$answer" != "y" ]; then
        die "Do not trigger build -> abort."
    fi

    print_info "Pushing tag 'prepare-$tag'."
    git push -f origin --no-follow-tags "$branch" "prepare-$tag"
}

function check_new_version() {
    local new_version="$1" # Reference to parent scoped variable.

    # Check that is a version.
    if [ "$(ci_container_mgr run --rm alpine/semver semver "$new_version" | tail -1)" != "$new_version" ]; then
        die "Your version '$new_version' is not sem. version compliant."
    fi

    if git tag --list "v*" | grep -qE "^v$new_version$"; then
        die "Git tag '$tag' already exists locally."
    fi

    # Get all remote versions.
    local remote_versions=()
    readarray -t remote_versions < \
        <(git ls-remote origin "regs/tags/v*" | cut -f 2 | sed "s@/refs/tags/v@@g")

    # shellcheck disable=SC2128
    if [ "${#remote_versions[@]}" = "0" ]; then
        # No version tags yet. Its ok.
        return 0
    fi

    if echo "${remote_versions[@]}" | grep "$new_version"; then
        die "Remote already contains version tag 'v$new_version'".
    fi

    # Sort the versions.
    # The top version must be the new one!
    latest=$(ci_container_mgr run --rm alpine/semver semver "${remote_versions[@]}" "$new_version" | tail -1)

    if [ "$latest" != "$new_version" ]; then
        die "Your version '$new_version' is not newer than the remote ones:" \
            "${remote_versions[@]}"
    fi
}

function main() {
    cd "$ROOT_DIR"

    local version="$1"

    local branch
    branch=$(git branch --show-current)

    if [ "$branch" != "$RELEASE_BRANCH" ] && [ "${FORCE_RELEASE:-}" != "true" ]; then
        die "Can only tag on 'main'. Use 'FORCE_RELEASE=true'."
    fi

    if ! git diff --quiet --exit-code; then
        die "You have changes on this branch."
    fi

    delete_prepare_tags

    check_new_version "$version"

    commit_version_file "$version"
    create_prepare_tag

    trigger_build "$branch"
}

main "$@"
