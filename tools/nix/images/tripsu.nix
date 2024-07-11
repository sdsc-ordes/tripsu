{
  pkgs,
  tripsu,
}:
pkgs.dockerTools.buildLayeredImage {
  name = "ghcr.io/sdsc-ordes/tripsu";
  tag = tripsu.version;

  contents = [tripsu];

  config = {
    Entrypoint = ["tripsu"];
    WorkingDir = "/";
    Labels = {
      "org.opencontainers.image.source" = "https://github.com/sdsc-ordes/tripsu";
      "org.opencontainers.image.description" = tripsu.meta.description;
      "org.opencontainers.image.license" = "Apache-2.0";
    };
  };
}
