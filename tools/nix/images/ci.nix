{
  pkgs,
  devShellDrv,
  ...
}: let
  version = "1.0.0"; # The version of these CI images.
  image_name = "ghcr.io/sdsc-order/rdf-protect";

  buildImage = type:
    pkgs.dockerTools.buildNixShellImage {
      name = image_name;
      tag = "ci-${type}-${version}";
      drv = devShellDrv;
    };
in rec {
  format = buildImage "format";
  lint = buildImage "lint";
  build = buildImage "build";
  test = buildImage "test";
  package = buildImage "package";
  deploy = buildImage "deploy";
}
