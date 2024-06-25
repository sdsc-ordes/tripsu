#!/usr/bin/env bash
# shellcheck disable=SC1090,SC1091
# shellcheck disable=SC2154,SC2086

function _print() {
    local color="$1"
    local flags="$2"
    local header="$3"
    shift 3

    local hasColor="0"
    if [ "${FORCE_COLOR:-}" != 1 ]; then
        [ -t 1 ] && hasColor="1"
    else
        hasColor="1"
    fi

    if [ "$hasColor" = "0" ] || [ "${LOG_COLORS:-}" = "false" ]; then
        local msg
        msg=$(printf '%b\n' "$@")
        msg="${msg//$'\n'/$'\n'   }"
        echo $flags -e "-- $header$msg"
    else
        local s=$'\033' e='[0m'
        local msg
        msg=$(printf "%b\n" "$@")
        msg="${msg//$'\n'/$'\n'   }"
        echo $flags -e "${s}${color}-- $header$msg${s}${e}"
    fi
}
function print_info() {
    _print "[0;94m" "" "" "$@"
}

function print_warning() {
    _print "[0;31m" "" "WARN: " "$@" >&2
}

function print_error() {
    _print "[0;31m" "" "ERROR: " "$@" >&2
}

function die() {
    print_error "$@"
    exit 1
}

function ci_is_running() {
    if [ "${CI:-}" = "true" ]; then
        return 0
    fi

    return 1
}

function ci_wrap_container() {
    local container="$1"
    shift 1
    local cmd=("$@")

    if [ "$OSTYPE" = "nixos" ]; then
        "${cmd[@]}"
    else
        ci_container_mgr_run_mounted "$(pwd)" "$container" "${cmd[@]}"
    fi
}

function ci_setup_githooks() {
    local installPrefix="${1:-$CI_BUILDS_DIR/githooks}"
    mkdir -p "$installPrefix"

    print_info "Install Githooks in '$installPrefix'."
    githooks-cli installer --non-interactive --prefix "$installPrefix"

    git hooks config enable-containerized-hooks --global --set
    git hooks config container-manager-types --global --set "podman,docker"

    print_info "Pull all shared Githooks repositories."
    git hooks shared update

    export CI_GITHOOKS_INSTALL_PREFIX="$installPrefix"
}

function ci_setup_nix() {
    local install_prefix="${1:-/usr/sbin}"

    print_info "Install Nix."
    apk add curl bash xz shadow
    sh <(curl -L https://nixos.org/nix/install) --daemon --yes
    cp /root/.nix-profile/bin/* "$install_prefix/"

    print_info "Enable Features for Nix."
    mkdir -p ~/.config/nix
    {
        echo "experimental-features = nix-command flakes"
        echo "accept-flake-config = true"
    } >~/.config/nix/nix.conf
}

# Run the container manager which is defined.
function ci_container_mgr() {
    if command -v podman &>/dev/null; then
        echo -e "Running podman as:\n$(printf "'%s' " "podman" "$@")" >&2
        podman "$@"
    else
        echo -e "Running docker as:\n$(printf "'%s' " "docker" "$@")"
        docker "$@"
    fi
}

# Define the container id `CI_JOB_CONTAINER_ID` where
# this job runs. Useful to mount same volumes as in
# this container with `ci_run_podman`.
function ci_container_mgr_setup() {
    export CONTAINER_HOST="unix://var/run/podman.sock"
    print_info "Container host: '$CONTAINER_HOST'"

    job_container_id=$(ci_container_mgr ps \
        --filter "label=com.gitlab.gitlab-runner.type=build" \
        --filter "label=com.gitlab.gitlab-runner.job.id=$CI_JOB_ID" \
        --filter "label=com.gitlab.gitlab-runner.project.id=$CI_PROJECT_ID" \
        --filter "label=com.gitlab.gitlab-runner.pipeline.id=$CI_PIPELINE_ID" \
        --format "{{ .ID }}") ||
        die "Could not find 'build' container for job id: '$CI_JOB_ID'."

    [ -n "$job_container_id" ] || die "Job id is empty."

    export CI_JOB_CONTAINER_ID="$job_container_id"
    print_info "Job container id: '$CI_JOB_CONTAINER_ID'"
}

function ci_container_mgr_login() {
    local user="$1"
    local token="$2"

    [ -n "$token" ] || die "Docker login token is empty"
    echo "$token" |
        ci_container_mgr login --password-stdin --username "$user" ||
        die "Could not log into docker."
}

# Run container mgr. In CI with volume mount from the
# current build container `CI_JOB_CONTAINER_ID`.
function ci_container_mgr_run() {
    if ci_is_running; then
        ci_container_mgr run --volumes-from "$CI_JOB_CONTAINER_ID" "$@"
    else
        ci_container_mgr run "$@"
    fi
}

function ci_container_mgr_run_mounted() {
    local repo workspace_rel in_cmd
    repo=$(git rev-parse --show-toplevel)
    workspace_rel=$(cd "$1" && pwd)
    workspace_rel=$(realpath --relative-to "$repo" "$workspace_rel")

    shift 1
    in_cmd=("$@")

    local mnt_args=()
    local cmd=()

    if ! ci_is_running; then
        cmd=("${in_cmd[@]}")
        mnt_args+=(-v "$repo:/repo")
        mnt_args+=(-w "/repo/$workspace_rel")
    else
        # Not needed to mount anything, since already existing
        # under the same path as `repo`.
        #
        # All `/repo` and `/workspace` paths in
        # command given are replaced with correct
        # paths to mounted volume in CI
        for arg in "${in_cmd[@]}"; do
            cmd+=("$(echo "$arg" |
                sed -E \
                    -e "s@/workspace@$workspace_rel@g" \
                    -e "s@/repo@$repo@g")")
        done

        mnt_args+=(-w "$repo/$workspace_rel")
    fi

    ci_container_mgr_run "${mnt_args[@]}" "${cmd[@]}"
}
