{
  pkgs,
  devShellDrv,
  ...
}: rec {
  # The base image.
  base = pkgs.dockerTools.buildNixShellImage {
    name = "ghcr.io/sdsc-order/rdf-protect";
    tag = "ci-nix-1.0.0";
    drv = devShellDrv;
  };

  format = base.override {tag = "ci-format-1.0.0";};
  lint = base.override {tag = "ci-lint-1.0.0";};
  build = base.override {tag = "ci-build-1.0.0";};
}
