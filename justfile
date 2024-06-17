set positional-arguments
set shell := ["bash", "-cue"]
comp_dir := justfile_directory()
root_dir := `git rev-parse --show-toplevel`

# General Variables:
# You can chose either "podman" or "docker"
container_mgr := "podman"

build *args:
    cd "{{root_dir}}" && cargo build "${@:1}"

watch:
    cd "{{root_dir}}" && cargo watch -x 'build'

format:
    cd "{{root_dir}}" && \
        {{container_mgr}} run -v "{{root_dir}}:/repo" -v "$(pwd):/workspace" -w "/workspace" \
    	instrumentisto/rust:nightly-alpine cargo fmt -- --config-path /repo
