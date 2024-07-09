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
    Labels = {
      "org.opencontainers.image.source" = "https://github.com/sdsc-ordes/rdf-protect";
      "org.opencontainers.image.description" = rdf-protect.meta.description;
      "org.opencontainers.image.license" = "Apache-2.0";
    };
  };
}
