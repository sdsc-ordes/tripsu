{pkgs, lib, rustPlatform, rootDir}:
      rustPlatform.buildRustPackage rec {
        inherit buildInputs nativeBuildInputs;

        name = "rdf-protect";
        src = rootDir;
        version = "1.0.0";

        cargoLock = {
          lockFile = "${rootDir}/Cargo.lock";
          allowBuiltinFetchGit = true;
        };

        preConfigure = ''
        '';

        postPatch = ''
        '';

        preFixup = lib.optionalString stdenv.isLinux ''
          patchelf \
            --add-needed "${pkgs.libGL}/lib/libEGL.so.1" \
            --add-needed "${pkgs.vulkan-loader}/lib/libvulkan.so.1" \
            $out/bin/wezterm-gui
        '';

        postInstall = ''
          mkdir -p $out/nix-support
          echo "${passthru.terminfo}" >> $out/nix-support/propagated-user-env-packages

          install -Dm644 assets/icon/terminal.png $out/share/icons/hicolor/128x128/apps/org.wezfurlong.wezterm.png
          install -Dm644 assets/wezterm.desktop $out/share/applications/org.wezfurlong.wezterm.desktop
          install -Dm644 assets/wezterm.appdata.xml $out/share/metainfo/org.wezfurlong.wezterm.appdata.xml

          install -Dm644 assets/shell-integration/wezterm.sh -t $out/etc/profile.d
          installShellCompletion --cmd wezterm \
            --bash assets/shell-completion/bash \
            --fish assets/shell-completion/fish \
            --zsh assets/shell-completion/zsh

          install -Dm644 assets/wezterm-nautilus.py -t $out/share/nautilus-python/extensions
        '';

        passthru = {
          terminfo =
            pkgs.runCommand "wezterm-terminfo"
            {
              nativeBuildInputs = [pkgs.ncurses];
            } ''
              mkdir -p $out/share/terminfo $out/nix-support
              tic -x -o $out/share/terminfo ${src}/termwiz/data/wezterm.terminfo
            '';
        };
      };
