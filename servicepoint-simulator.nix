{
  craneLib,
  pkgs,
}:
let
  commonArgs = {
    src = craneLib.cleanCargoSource ./.;
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
          libxcursor
          libxrandr
          libxi
          libx11
          libx11.dev
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
  };
in
craneLib.buildPackage (
  commonArgs
  // {
    cargoArtifacts = craneLib.buildDepsOnly commonArgs;

    postInstall = ''
      wrapProgram $out/bin/servicepoint-simulator \
        --suffix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath commonArgs.buildInputs}
    '';
  }
)
