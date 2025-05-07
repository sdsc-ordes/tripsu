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
benchmark *args:
    cd {{root_dir}} && \
        just nix-develop bench bash ./tools/bench/benchmark.sh {{args}}

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
## ============================================================================
