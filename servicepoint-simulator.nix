{
  naersk',
  pkgs,
  nix-filter,
}:
naersk'.buildPackage rec {
  src = nix-filter.lib.filter {
    root = ./.;
    include = [
      ./Cargo.toml
      ./Cargo.lock
      ./src
      ./Web437_IBM_BIOS.woff
      ./README.md
      ./LICENSE
    ];
  };
  nativeBuildInputs = with pkgs; [
    pkg-config
    makeWrapper
  ];
  strictDeps = true;
  buildInputs =
    with pkgs;
    [
      xe
      xz

      roboto
    ]
    ++ lib.optionals pkgs.stdenv.isLinux (
      with pkgs;
      [
        # gpu
        libGL
        vulkan-headers
        vulkan-loader
        vulkan-tools
        vulkan-tools-lunarg
        vulkan-extension-layer
        vulkan-validation-layers

        # keyboard
        libxkbcommon

        # font loading
        fontconfig
        freetype

        # WINIT_UNIX_BACKEND=wayland
        wayland

        # WINIT_UNIX_BACKEND=x11
        xorg.libXcursor
        xorg.libXrandr
        xorg.libXi
        xorg.libX11
        xorg.libX11.dev
      ]
    )
    ++ lib.optionals pkgs.stdenv.isDarwin (
      with pkgs.darwin.apple_sdk.frameworks;
      [
        Carbon
        QuartzCore
        AppKit
      ]
    );

  postInstall = ''
    wrapProgram $out/bin/servicepoint-simulator \
      --suffix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath buildInputs}
  '';
}
