{
  pkgs,
  lib,
  rustToolchain,
  rootDir,
  ...
}: let
  rustPlatform = pkgs.makeRustPlatform {
    cargo = rustToolchain;
    rustc = rustToolchain;
  };
in
  rustPlatform.buildRustPackage {
    name = "rdf-protect";
    src = rootDir;
    version = "1.0.0";

    cargoLock = {
      lockFile = rootDir + "/Cargo.lock";
    };

    meta = {
      description = "A simple Rust CLI tool to protect sensitive values in RDF triples through pseudonymization";
      homepage = "https://github.com/sdsc-ordes/rdf-protect";
      license = lib.licenses.asl20;
      maintainers = ["gabyx" "cmdoret"];
    };
  }
