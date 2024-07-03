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

function ci_is_release() {
    if [ "${CI_IS_RELEASE:-}" = "true" ]; then
        return 0
    fi

    return 1
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

function ci_container_mgr_login() {
    local user="$1"
    local token="$2"

    [ -n "$token" ] || die "Docker login token is empty"
    echo "$token" |
        ci_container_mgr login --password-stdin --username "$user" ||
        die "Could not log into docker."
}
