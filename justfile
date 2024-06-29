set positional-arguments
set shell := ["bash", "-cue"]
root_dir := `git rev-parse --show-toplevel`

# General Variables:
# You can chose either "podman" or "docker".
container_mgr := "podman"

# Deterministic steps such as `lint`, `format`
# will run
use_container := ""

# Default recipe to list all recipes.
default:
  just --list

# Enter a Nix development shell.
nix-develop *args:
    cd "{{root_dir}}" && \
    cmd=("$@") && \
    { [ -n "${cmd:-}" ] || cmd=("zsh"); } && \
    nix develop ./tools/nix#default --command "${cmd[@]}"

## Standard stuff =============================================================
# Build the executable.
build *args:
    cd "{{root_dir}}" && cargo build "${@:1}"

# Watch source and continuously build the executable.
watch:
    cd "{{root_dir}}" && cargo watch -x 'build'

# Run the executable.
run:
    cd "{{root_dir}}" && cargo run "${@:1}"

# Run the tests.
test:
    cd "{{root_dir}}" && cargo test "${@:1}"

# Format the code.
format *args:
    cd "{{root_dir}}" && \
        "{{root_dir}}/tools/format-rust.sh" {{args}}

# Format all files.
format-general *args:
    # Not implemented yet.
    # That should run all hooks which are configured by Githooks.
    true

# Lint all code.
lint *args:
    cd "{{root_dir}}" && \
        "{{root_dir}}/tools/lint-rust.sh" {{args}}

# Lint all code (undefined behavior).
lint-ub *args:
    cd "{{root_dir}}" && \
        "{{root_dir}}/tools/lint-ub-rust.sh" {{args}}

# Create a new release for version `version` by
# updating the version file and
# triggering the release workflow.
release version:
    cd "{{root_dir}}" && \
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

# Upload all images for CI.
upload-ci-images:
    cd "{{root_dir}}" && \
        .gitlab/scripts/upload-images.sh
## ============================================================================
