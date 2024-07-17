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

function ci_setup_git() {
    git config --global user.name "SDSC CI"
    git config --global user.email "ci@sdsc.ethz.ch"
}

function ci_setup_nix() {
    # local install_prefix="${1:-/usr/sbin}"

    # print_info "Install Nix."
    #
    # apk add curl git bash xz shadow sudo -t deps
    # sh <(curl -L https://nixos.org/nix/install) --no-daemon --yes || die "Could not install Nix."
    # cp /root/.nix-profile/bin/* "$install_prefix/"
    # apk del deps

    print_info "Enable Features for Nix."
    mkdir -p ~/.config/nix
    {
        echo "experimental-features = nix-command flakes"
        echo "accept-flake-config = true"
    } >~/.config/nix/nix.conf

}

function ci_setup_github_workarounds() {
    # Hacks to get the mounted nodejs by github actions work as its dynamically linked
    # https://github.com/actions/checkout/issues/334#issuecomment-716068696
    nix build --no-link 'nixpkgs#stdenv.cc.cc.lib' 'nixpkgs#glibc'
    local ld_path link
    ld_path="$(nix path-info 'nixpkgs#stdenv.cc.cc.lib')/lib"
    echo "LD_LIBRARY_PATH=$ld_path" >"/container-setup/.ld-library-path"
    link="$(nix path-info 'nixpkgs#glibc' --recursive | grep glibc | grep -v bin)/lib64" || die "Could not get link."
    ln -s "$link" /lib64
}

function ci_setup_cachix {
    local name="$1"
    local token="$2"

    print_info "Setup cachix binary cache."

    [ -n "$name" ] ||
        die "Cachix cache name is empty."
    [ -n "$token" ] ||
        die "Cachix token is empty."

    cachix authtoken --stdin < <(echo "$token")
    cachix use --mode user-nixconf "$name" ||
        die "Could not setup cachix cache '$name'."

    print_info "Cachix binary cache set up."
}

# Run the container manager which is defined.
# in env. variable `CONTAINER_MGR`
# (by default `podman` if existing).
function ci_container_mgr() {
    local mgr="${CONTAINER_MGR:-podman}"

    if command -v "$mgr" &>/dev/null; then
        print_info "Running '$mgr' as:\n$(printf "'%s' " "podman" "$@")" >&2
        "$mgr" "$@"
    else
        print_info "Running docker as:\n$(printf "'%s' " "docker" "$@")"
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
