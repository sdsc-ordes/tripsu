#!/usr/bin/env bash
# shellcheck disable=SC1090,SC1091
#
# Push image to the registry.

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
. "$ROOT_DIR/tools/general.sh"

cd "$ROOT_DIR"

function main() {

    local username=${REGISTRY_USERNAME:-$USERNAME}
    local password=${REGISTRY_PASSWORD:-$PASSWORD}

    if [ -z "$username" ] || [ -z "$password" ]; then
        die "'USERNAME' or 'PASSWORD' env. variables not set."
    fi

    if ! ci_is_running; then
        die "This script should only be executed in CI"
    fi

    local image_names=()
    readarray -t image_names < <(cd build/image && find ./ -name "*.tar.gz")

    for image_name in "${image_names[@]}"; do

        image_path="$ROOT_DIR/build/image/$image_name"
        image_name=${image_name%.tar.gz} # Split `.tar.gz` from end.
        image_name=${image_name#./}      # Split `./` from front.
        image_name=${image_name/|/:}     # Replace `|` with `:`.

        print_info "Uploading image: '$image_name' in '$image_path'."

        print_info "Read the image from file '$image_path' and " \
            "directly push to registry '$image_name'."

        skopeo \
            --insecure-policy \
            copy \
            --dest-username "$(cat <(echo "$username"))" \
            --dest-password "$(cat <(echo "$password"))" \
            "docker-archive://$image_path" \
            "docker://$image_name"

    done
}

main "$@"
