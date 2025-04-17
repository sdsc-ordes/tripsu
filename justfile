set positional-arguments
set shell := ["bash", "-cue"]
root_dir := `git rev-parse --show-toplevel`
flake_dir := root_dir / "tools/nix"

# General Variables:
# You can chose either "podman" or "docker".
container_mgr := "podman"

# Default recipe to list all recipes.
default:
  just --list

# Enter the default Nix development shell.
develop *args:
    just nix-develop default "$@"

# Enter the CI Nix development shell for benchmarking.
develop-bench *args:
    just nix-develop bench "$@"

# Enter the CI Nix development shell.
ci *args:
    just nix-develop ci "$@"

# Enter a Nix development shell.
[private]
nix-develop *args:
    #!/usr/bin/env bash
    set -eu
    cd "{{root_dir}}"
    shell="$1"; shift 1;
    args=("$@") && [ "${#args[@]}" != 0 ] || args="$SHELL"
    nix develop --accept-flake-config \
        "{{flake_dir}}#$shell" \
        --command "${args[@]}"

## Standard stuff =============================================================
# Format the code.
format *args:
    nix run --accept-flake-config {{flake_dir}}#treefmt -- "$@"

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
    nix develop --profile "$profile/dev" {{flake_dir}}#ci --command true
    cachix push "$CACHIX_CACHE_NAME" "$profile/dev"
    rm -rf "$profile"

    # Cache flake inputs.
    nix flake archive {{flake_dir}} --json \
      | jq -r '.path,(.inputs|to_entries[].value.path)' \
      | cachix push "$CACHIX_CACHE_NAME"


# Upload all images for CI (local machine)
upload-ci-images:
    cd "{{root_dir}}" && \
        CONTAINER_MGR="{{container_mgr}}" \
        tools/ci/upload-ci-images.sh
## ============================================================================
