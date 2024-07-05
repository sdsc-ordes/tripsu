#!/usr/bin/env bash
# shellcheck disable=SC1090,SC1091
set -e
set -u

DIR=$(cd -- "$(dirname -- "$0")" &>/dev/null && pwd)
ROOT_DIR=$(git rev-parse --show-toplevel)
. "$ROOT_DIR/tools/general.sh"

cd "$ROOT_DIR"

function build_ci_image() {
    local image_type="$1"
    local repository="$2"
    local version="$3"
    local tag="$image_type-$version"

    local image_name="$repository:$tag"

    print_info "Building image '$image_name'."

    ci_container_mgr build -f "$container_file" \
        --target "$image_type" \
        -t "$image_name" \
        . || die "Could not build image."

    ci_container_mgr push "$image_name" || die "Could not upload image."
}

tag="${1:-1.0.0}"
repository="${2:-ghcr.io/sdsc-ordes/rdf-protect}"
container_file="$DIR/../images/Containerfile"

if [ "${CI:-}" = "true" ]; then
    ci_container_mgr_login "$DOCKER_REPOSITORY_READ_USERNAME" "$DOCKER_REPOSITORY_READ_TOKEN"
fi

readarray -t images < <(grep -E "as ci-.*" "$container_file" | sed -E 's@.*as (ci-.*)$@\1@g')
for image in "${images[@]}"; do
    build_ci_image "$image" "$repository" "$tag"
done
