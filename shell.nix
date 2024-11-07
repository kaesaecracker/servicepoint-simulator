{
  pkgs ? import <nixpkgs> { },
}:
let
  rust-toolchain = pkgs.symlinkJoin {
    name = "rust-toolchain";
    paths = with pkgs; [
      rustc
      cargo
      rustPlatform.rustcSrc
      rustfmt
      clippy
      cargo-expand
    ];
  };
in
pkgs.mkShell {
  nativeBuildInputs = with pkgs.buildPackages; [
    rust-toolchain

    pkg-config
    xe
    lzma

    # linux x11 / wayland
    libxkbcommon
    #xorg.libxkbfile
    wayland
  ];

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
    with pkgs;
    [
      wayland
      libxkbcommon
    ]
  );
}
