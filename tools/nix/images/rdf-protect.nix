{
  pkgs,
  rdf-protect,
}:
pkgs.dockerTools.buildImage {
  name = "ghcr.io/sdsc-order/rdf-protect";
  tag = rdf-protect.version;

  copyToRoot = pkgs.buildEnv {
    name = "image-root";
    paths = [rdf-protect];
    pathsToLink = ["/bin"];
  };

  config = {
    Cmd = ["/bin/rdf-protect"];
    WorkingDir = "/";
  };
}
