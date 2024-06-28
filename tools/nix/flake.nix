{
  description = "rdf-protect";

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
    self,
    nixpkgs,
    nixpkgsStable,
    flake-utils,
    rust-overlay,
    ...
  } @ inputs: let
    rootDir = ./. + "../../..";
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

        # Things needed only at compile-time.
        nativeBuildInputsBasic = with pkgs; [
          findutils
          coreutils
          bash
          zsh
          curl
          git
          jq

          podman
        ];

        # Things needed only at compile-time.
        nativeBuildInputsDev = with pkgs; [
          rustToolchain
          cargo-watch
          just
        ];

        # Things needed at runtime.
        buildInputs = [];

        # The package of this CLI tool.
        # The global version for rdf-protect.
        # This is gonna get tooled later.
        rdf-protect-version = "1.0.0";
        rdf-protect = (import ./pkgs/rdf-protect.nix) {
          inherit rootDir rustToolchain pkgs lib;
          version = rdf-protect-version;
        };
      in
        with pkgs; rec {
          devShells = rec {
            default = mkShell {
              inherit buildInputs;
              nativeBuildInputs = nativeBuildInputsBasic ++ nativeBuildInputsDev;
            };

            ci = default;
          };

          packages = {
            images = {
              ci = (import ./images/ci.nix) {
                inherit pkgs;
                devShellDrv = devShells.ci;
              };

              rdf-protect = (import ./images/rdf-protect.nix) {
                inherit pkgs rdf-protect;
              };
            };
          };
        }
    );
}
