{
  pkgs,
  rdf-protect,
}:
pkgs.dockerTools.buildImage {
  name = "ghcr.io/sdsc-ordes/rdf-protect";
  tag = rdf-protect.version;

  copyToRoot = pkgs.buildEnv {
    name = "image-root";
    paths = [rdf-protect];
    pathsToLink = ["/bin"];
  };

  config = {
    Entrypoint = ["/bin/rdf-protect"];
    WorkingDir = "/";
  };
}
