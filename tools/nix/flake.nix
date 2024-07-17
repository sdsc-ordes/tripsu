{
  description = "tripsu";

  nixConfig = {
    substituters = [
      # Add here some other mirror if needed.
      "https://cache.nixos.org/"
    ];
    extra-substituters = [
      # Nix community's cache server
      "https://nix-community.cachix.org"
    ];
    extra-trusted-public-keys = [
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };

  inputs = {
    # Nixpkgs
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    # You can access packages and modules from different nixpkgs revs
    # at the same time. Here's an working example:
    nixpkgsStable.url = "github:nixos/nixpkgs/nixos-23.11";
    # Also see the 'stable-packages' overlay at 'overlays/default.nix'.

    flake-utils.url = "github:numtide/flake-utils";

    # The Rust overlay to include the latest toolchain.
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = {
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }: let
    # This is string (without toString it would be a `path` which is put into the store)
    rootDir = toString ./. + "../../..";
  in
    flake-utils.lib.eachDefaultSystem
    # Creates an attribute map `{ devShells.<system>.default = ...}`
    # by calling this function:
    (
      system: let
        overlays = [(import rust-overlay)];

        # Import nixpkgs and load it into pkgs.
        # Overlay the rust toolchain
        lib = nixpkgs.lib;
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Set the rust toolchain from the `rust-toolchain.toml`.
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ../../rust-toolchain.toml;

        # Basic Packages.
        nativeBuildInputsBasic = with pkgs; [
          findutils
          coreutils
          gnugrep
          bash
          curl
          git
          git-lfs
          jq
          just

          # Nix binary cache.
          cachix
        ];

        # Packges for development.
        nativeBuildInputsDev = with pkgs; [
          # General build tooling.
          rustToolchain
          cargo-watch

          # Uploading images.
          skopeo

          # Modifying toml files.
          dasel
        ];

        # Things needed at runtime.
        buildInputs = [];

        # The package of this CLI tool.
        tripsu = (import ./pkgs/tripsu.nix) {
          inherit rootDir rustToolchain pkgs lib;
        };
      in
        with pkgs; rec {
          devShells = {
            # Local development environment.
            default = mkShell {
              inherit buildInputs;
              nativeBuildInputs = nativeBuildInputsBasic ++ nativeBuildInputsDev;
            };

            # CI environment.
            ci = mkShell {
              inherit buildInputs;
              nativeBuildInputs = nativeBuildInputsBasic ++ nativeBuildInputsDev;

              # Due to some weird handling of TMPDIR inside containers:
              # https://github.com/NixOS/nix/issues/8355
              # We have to reset the TMPDIR to make `nix build` work inside
              # a development shell.
              # Without `nix develop` it works.
              shellHook = "unset TMPDIR";
            };
          };

          packages = {
            # Package of this repo.
            tripsu = tripsu;

            # Packages for CI.
            ci = {
              # CI bootstrapping packages:
              # add some basic utils to the Nix store for CI.
              bootstrap = pkgs.buildEnv {
                name = "ci-bootstrap";
                paths = nativeBuildInputsBasic;
              };
            };

            # Container Images.
            images = {
              ci = (import ./images/ci.nix) {
                inherit pkgs;
                devShellDrv = devShells.default;
              };

              tripsu = (import ./images/tripsu.nix) {
                inherit pkgs tripsu;
              };
            };
          };
        }
    );
}
