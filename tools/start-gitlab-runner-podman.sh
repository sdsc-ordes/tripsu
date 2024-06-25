#!/usr/bin/env bash
# shellcheck disable=SC1090,SC1091,SC2015
#
# Create a gitlab runner (docker executor)
# by first visiting `CI/CD Settings` page and
# creating a `linux` runner which gives you a `<token>` needed
# for this script.
#
# This creates a container which runs the Gitlab runner
# which will execute jobs over the `podman` executor.
#
# This container is based on [`pipglr`](https://gitlab.com/qontainers/pipglr)
# which uses two container volumes (`pipglr-storage` and `pipglr-cache`)
# which are attached and contains
# `podman` (user `podman`) and `gitlab-runner` (user `runner`)
# to have a rootless container experience which provides much more security.
# The two volumes contain the images created/built by CI. The volumes can safely
# be wiped if space is needed.
#
# The podman socket created inside this container
# will be mounted to each job container the `gitlab-runner` creates.
# This makes also use of caching directly possible which is cool.
#
# Usage:
# ```shell
# start-gitlab-runner-docker.sh [--force] [<token>]
# ```
# Read token from stdin.
# ```shell
# start-gitlab-runner-docker.sh [--force] -
#
# Usage in Pipeline:
#
# A job which uses `podman` (linked to podman) to run/build images.
# The gitlab-runner cannot serve `services` statements as in the
# `start-gitlab-runner-docker.sh` (uses `docker`
# `--link` which is anyway deprecated)
#
# ```yaml
# podman-remote-run-build:
#   image: quay.io/podman/stable:latest
#   variables:
#     CONTAINER_HOST: unix://var/run/docker.sock
#   script:
#     - podman info
#     - podman run alpine:latest cat /etc/os-release
#     - podman build -f Dockerfile .
# ```
#
# The following (custom build image) also works:
#
# ```yaml
# podman-remote-alpine-run-build:
#   image: alpine:latest
#   variables:
#     CONTAINER_HOST: unix://var/run/docker.sock
#   script:
#     - apk add podman
#     - podman info
#     - podman run alpine:latest cat /etc/os-release
#     - podman build -f Dockerfile .
# ```

set -e
set -u

ROOT=$(git rev-parse --show-toplevel)
. "$ROOT/tools/general.sh"

force="false"
max_jobs=4
config_dir="$ROOT/.gitlab/local/config"
runner_name="gitlab-runner-md2pdf-podman"
cores=$(grep "^cpu\\scores" /proc/cpuinfo | uniq | cut -d ' ' -f 3)
# image="registry.gitlab.com/qontainers/pipglr:latest"
image="pipglr:dev-latest-alpine"

function clean_up() {
    if [ -f "$config_dir/config.toml" ]; then
        rm -rf "$config_dir/config.toml"
    fi
}

trap clean_up EXIT
function modify_config() {
    local key="$1"
    local value="$2"
    local type="${3:-json}"

    podman run --rm -v "$config_dir/config.toml:/config.toml" \
        "ghcr.io/tomwright/dasel" put -f /config.toml \
        -t "$type" \
        -s "$key" \
        -v "$value" ||
        die "Could not set gitlab runner config key '$key' to '$value'"
}

function register_runner() {
    print_info "Registering gitlab-runner ..."
    local token="${1:-}"

    if [ "$token" = "-" ] || [ -z "$token" ]; then
        read -rs -p "Enter Gitlab Runner Token: " token ||
            die "Could not read token from stdin."
    fi

    podman secret rm REGISTRATION_TOKEN &>/dev/null || true
    echo "$token" | podman secret create REGISTRATION_TOKEN - ||
        die "Could not set registration token secret."

    # Register Gitlab runner.
    (cd "$config_dir" &&
        touch config.toml &&
        podman container runlabel register "$image") ||
        die "Could not register gitlab-runner."

    # Modify Gitlab runner config.
    modify_config ".concurrent" "$max_jobs"
    modify_config ".runners.first().docker.pull_policy" \
        '["always"]'
    modify_config ".runners.first().docker.volumes.append()" \
        "/home/runner/podman.sock:/var/run/podman.sock:rw" string

    # Add an auxiliary volume `auxvol`.
    modify_config ".runners.first().docker.volumes.append()" \
        "auxvol:/auxvol" string

    modify_config ".runners.first().pre_build_script" \
        "echo 'Prebuild'\\nenv" string

    podman secret rm config.toml &>/dev/null || true
    podman secret create config.toml "$config_dir/config.toml" ||
        die "Could not create config.toml secret."

    print_info "Config file:" \
        "$(sed 's/token\s*=.*/token = ***/g' "$config_dir/config.toml")"

    rm "$config_dir/config.toml"
}

function assert_volumes() {
    print_info "Asserting needed volumes ..."

    local volumes
    volumes=$(podman volume list --format="{{ .Name }}") ||
        die "Could not get volumes"

    if ! echo "$volumes" | grep -q "pipglr-storage"; then
        podman container runlabel setupstorage "$image"
    fi

    if ! echo "$volumes" | grep -q "pipglr-cache"; then
        podman container runlabel setupcache "$image"
    fi
}

function start_runner() {
    print_info "Start runner '$runner_name' ..."

    # Run the Gitlab runner. We cannot user `podman container runlabel run "$image"`
    # because we need to set some cpu constraints.
    podman run -dt --name "$runner_name" \
        --cpus "$cores" \
        --secret config.toml,uid=1001,gid=1001 \
        -v pipglr-storage:/home/podman/.local/share/containers \
        -v pipglr-cache:/cache \
        --systemd true --privileged \
        --device /dev/fuse "$image"

    podman exec -it --user root "$runner_name" \
        bash -c "mkdir -p /etc/containers;
                 cp /usr/share/containers/seccomp.json /etc/containers/seccomp.json"
}

function create() {
    rm -rf "$config_dir" >/dev/null || true
    mkdir -p "$config_dir"

    register_runner "$@"
    assert_volumes

    start_runner
}

function stop() {
    if is_running; then
        print_info "Stop runner '$runner_name' ..."
        podman stop "$runner_name"

    fi

    if is_exited; then
        # shellcheck disable=SC2046
        podman rm $(podman ps -a -q)
    fi
}

function is_running() {
    [ "$(podman inspect -f '{{.State.Status}}' "$runner_name" 2>/dev/null || true)" = 'running' ] || return 1
    return 0
}

function is_exited() {
    [ "$(podman inspect -f '{{.State.Status}}' "$runner_name" 2>/dev/null || true)" = 'exited' ] || return 1
    return 0
}

if [ "${1:-}" = "--force" ]; then
    force="true"
    shift 1
fi

if [ "$force" = "true" ]; then
    stop
fi

if ! is_running; then
    create "$@"
else
    print_info "Gitlab runner '$runner_name' is already running. Restart it."
    podman restart "$runner_name" ||
        die "Could not restart gitlab runner '$runner_name'."
fi
