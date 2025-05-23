{
  pkgs,
  lib,
  rustToolchain,
  rootDir,
  ...
}:
let
  rustPlatform = pkgs.makeRustPlatform {
    cargo = rustToolchain;
    rustc = rustToolchain;
  };

  cargoFile = rootDir + "/Cargo.toml";
  lockFile = rootDir + "/Cargo.lock";
in
rustPlatform.buildRustPackage {
  name = "tripsu";
  src = rootDir;

  version = (lib.importTOML cargoFile).package.version;

  cargoLock = {
    inherit lockFile;
  };

  meta = {
    description = "A simple Rust CLI tool to protect sensitive values in RDF triples through pseudonymization";
    homepage = "https://github.com/sdsc-ordes/tripsu";
    license = lib.licenses.asl20;
    maintainers = [
      "gabyx"
      "cmdoret"
    ];
  };
}
