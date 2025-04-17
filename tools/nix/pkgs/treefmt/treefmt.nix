{ pkgs, ... }:
{
  # Used to find the project root
  # We use `LICENSE.md` instead of `.git/config`
  # because that does not work with Git worktree.
  projectRootFile = "LICENSE.md";

  settings.global.excludes = [ "external/*" ];

  # Markdown, JSON, YAML, etc.
  programs.prettier.enable = true;

  programs.rustfmt = {
    enable = true;
    # package = ;
  };

  # Shell.
  programs.shfmt = {
    enable = true;
    indent_size = 4;
  };

  programs.shellcheck.enable = true;
  settings.formatter.shellcheck = {
    options = [
      "-e"
      "SC1091"
    ];
  };

  # Nix.
  programs.nixfmt.enable = true;

  # Typos.
  programs.typos.enable = false;
}
