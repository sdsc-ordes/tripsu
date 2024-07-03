{
  pkgs,
  rdf-protect,
}:
pkgs.dockerTools.buildLayeredImage {
  name = "ghcr.io/sdsc-ordes/rdf-protect";
  tag = rdf-protect.version;

  contents = [rdf-protect];

  config = {
    Entrypoint = ["rdf-protect"];
    WorkingDir = "/";
  };
}
