{
  pkgs,
  tripsu,
}:
pkgs.dockerTools.buildLayeredImage {
  name = "ghcr.io/sdsc-ordes/tripsu";
  tag = tripsu.version;

  contents = [tripsu];

  fakeRootCommands = ''
    ${pkgs.dockerTools.shadowSetup}
    groupadd -r non-root
    useradd -r -g non-root non-root
    mkdir -p /workspace
    chown non-root:non-root /workspace
  '';
  enableFakechroot = true;

  config = {
    Entrypoint = ["tripsu"];
    WorkingDir = "/workspace";
    Labels = {
      "org.opencontainers.image.source" = "https://github.com/sdsc-ordes/tripsu";
      "org.opencontainers.image.description" = tripsu.meta.description;
      "org.opencontainers.image.license" = "Apache-2.0";
    };
    User = "non-root";
  };
}
