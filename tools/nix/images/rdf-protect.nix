{
  pkgs,
  rdf-protect,
}:
pkgs.dockerTools.buildImage {
  name = "ghcr.io/sdsc-order/rdf-protect";
  tag = rdf-protect.version;

  config = {
    Cmd = "${rdf-protect}/bin/rdf-protect";
  };
}
