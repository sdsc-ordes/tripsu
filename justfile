set positional-arguments
set shell := ["bash", "-cue"]
comp_dir := justfile_directory()
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
    cd "{{comp_dir}}" && \
        "{{root_dir}}/tools/format-rust.sh" {{args}}

# Format all files other files.
format-general *args:
    # Not implemented yet.
    true

# Lint all code.
lint *args:
    cd "{{comp_dir}}" && \
        "{{root_dir}}/tools/lint-rust.sh" {{args}}
## ============================================================================


## CI stuff ===================================================================
# Enter a Nix development shell for CI.
nix-develop-ci:
    cd "{{root_dir}}" && nix develop ./tools/nix#default --command "$@"

# Build the nix package into the folder `package` (first argument).
nix-package *args:
    dir="${1:-package}" && \
        cd "{{root_dir}}" && \
        nix build "./tools/nix#rdf-protect" \
        --out-link "$dir" \
        "${@:2}"

# Upload all images for CI.
upload-ci-images:
    cd "{{root_dir}}" && \
        .gitlab/scripts/upload-images.sh
## ============================================================================
