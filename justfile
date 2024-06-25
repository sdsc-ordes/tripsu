set positional-arguments
set shell := ["bash", "-cue"]
comp_dir := justfile_directory()
root_dir := `git rev-parse --show-toplevel`

# General Variables:
# You can chose either "podman" or "docker".
container_mgr := "podman"

# Enter a Nix development shell.
nix-develop shell="zsh":
    cd "{{root_dir}}" && nix develop ./tools/nix#default --command zsh


# Build the executable.
build *args:
    cd "{{root_dir}}" && cargo build "${@:1}"

# Watch source and continuously build the executable.
watch:
    cd "{{root_dir}}" && cargo watch -x 'build'

# Run the executable.
run:
    cd "{{root_dir}}" && cargo run "${@:1}"

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

# Upload all images for CI.
upload-ci-images:
    cd "{{root_dir}}" && \
        .gitlab/scripts/upload-images.sh

# Enter a Nix development shell for CI.
nix-develop-ci:
    cd "{{root_dir}}" && nix develop ./tools/nix#default --command "$@"
