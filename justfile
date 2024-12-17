set positional-arguments
set shell := ["bash", "-cue"]
root_dir := `git rev-parse --show-toplevel`

# General Variables:
# You can chose either "podman" or "docker".
container_mgr := "podman"

# Default recipe to list all recipes.
default:
  just --list

# Enter a Nix development shell.
nix-develop *args:
    cd "{{root_dir}}" && \
    cmd=("$@") && \
    { [ -n "${cmd:-}" ] || cmd=("zsh"); } && \
    nix develop ./tools/nix#default --accept-flake-config --command "${cmd[@]}"

nix-develop-ci *args:
    #!/usr/bin/env bash
    set -eu
    cd "{{root_dir}}"
    cachix watch-exec "$CACHIX_CACHE_NAME" -- \
        nix develop ./tools/nix#ci --accept-flake-config --command "$@"

# Enter nix development shell for benchmarking.
nix-develop-bench *args:
    cd "{{root_dir}}" && \
    cmd=("$@") && \
    { [ -n "${cmd:-}" ] || cmd=("zsh"); } && \
    nix develop ./tools/nix#bench --command "${cmd[@]}"

## Standard stuff =============================================================
# Format the code.
format *args:
    cd "{{root_dir}}" && \
        "{{root_dir}}/tools/format-rust.sh" {{args}}

# Lint all code.
lint *args:
    cd "{{root_dir}}" && \
        "{{root_dir}}/tools/lint-rust.sh" {{args}}

# Build the executable.
build *args:
    cd "{{root_dir}}" && cargo build "${@:1}"

# Run the tests.
test:
    cd "{{root_dir}}" && cargo test "${@:1}"


## Development functionality ==================================================
# Watch source and continuously build the executable.
watch:
    cd "{{root_dir}}" && cargo watch -x 'build'

# Run the executable.
run:
    cd "{{root_dir}}" && cargo run "${@:1}"

# Create a new release by version bumping.
# Usage:
# ```shell
#    just release <sem-version>
# ```
# by updating the version file and triggering the release workflow.
release version:
    cd "{{root_dir}}" && \
        CONTAINER_MGR="{{container_mgr}}" \
        "{{root_dir}}/tools/release.sh" "{{version}}"
## ============================================================================


## CI stuff ===================================================================
# Build the nix package into the folder `package` (first argument).
nix-package *args:
    cd "{{root_dir}}" && \
       "./tools/build-package.sh" "$@"

# Build the Docker image with Nix (distroless by default!).
nix-image *args:
    cd "{{root_dir}}" && \
       "./tools/build-image.sh" "$@"


# Upload the dev shell to the Nix cache.
nix-cache-upload-shell:
    #!/usr/bin/env bash
    set -eu
    cd "{{root_dir}}"

    profile=./dev-profile
    mkdir -p "$profile"

    # Cache development shell.
    nix develop --profile "$profile/dev" ./tools/nix#ci --command true
    cachix push "$CACHIX_CACHE_NAME" "$profile/dev"
    rm -rf "$profile"

    # Cache flake inputs.
    nix flake archive ./tools/nix --json \
      | jq -r '.path,(.inputs|to_entries[].value.path)' \
      | cachix push "$CACHIX_CACHE_NAME"


# Upload all images for CI (local machine)
upload-ci-images:
    cd "{{root_dir}}" && \
        CONTAINER_MGR="{{container_mgr}}" \
        tools/ci/upload-ci-images.sh
## ============================================================================
