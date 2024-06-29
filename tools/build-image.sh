#!/usr/bin/env bash
# shellcheck disable=SC1090,SC1091
#
# Build the Nix container image.

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
. "$ROOT_DIR/tools/general.sh"

VERSION_FILE=$(ci_nix_image_version_file)

function clean_up() {
    if ! ci_is_running; then
        # Never change the version file, only
        # on explicit `just release`.
        print_info "Restoring '$VERSION_FILE'."
        git restore "$VERSION_FILE" || true
    fi
}

trap clean_up EXIT

function main() {
    args=("$@")

    if ! ci_is_running || ! ci_is_release; then
        # Define the image version from Git SHA
        version="0.0.0+$(git rev-parse --short=10 HEAD)"

        # Write the temporary version file (gets restored...)
        temp=$(mktemp)
        jq ".version |= \"$version\"" "$VERSION_FILE" >"$temp"
        mv "$temp" "$VERSION_FILE"
    else
        # When CI and in Release, the requested version must match.
        version=$(jq ".version" "$VERSION_FILE")

        release_version=${GITHUB_REF##*prepare-v}

        if [ "$version" != "$release_version" ]; then
            die "The version '$version' in '$VERSION_FILE' does not corresponds" \
                "with the version '$release_version' to build." \
                "Update the version file to align!" \
                "Nix is pure and cannot rely on Git tags to" \
                "get the version from."
        fi
    fi

    image_name=$(nix eval --raw "./tools/nix#images.rdf-protect.imageName")
    image_tag=$(nix eval --raw "./tools/nix#images.rdf-protect.imageTag")
    dir="build/image/$image_name|$image_tag.tar.gz"

    cd "$ROOT_DIR"

    print_info "Building image '$dir'."
    nix build "./tools/nix#images.rdf-protect" \
        --out-link "$dir" "${args[@]}"
}

main "$@"
