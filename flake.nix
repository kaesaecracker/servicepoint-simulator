{
  description = "Flake for servicepoint-simulator";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
    nix-filter.url = "github:numtide/nix-filter";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      naersk,
      nix-filter,
    }:
    let
      lib = nixpkgs.lib;
      supported-systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      forAllSystems = lib.genAttrs supported-systems;
      make-rust-toolchain-core =
        pkgs:
        pkgs.symlinkJoin {
          name = "rust-toolchain-core";
          paths = with pkgs; [
            rustc
            cargo
            rustPlatform.rustcSrc
          ];
        };
    in
    rec {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages."${system}";
          rust-toolchain-core = make-rust-toolchain-core pkgs;
          naersk' = pkgs.callPackage naersk {
            cargo = rust-toolchain-core;
            rustc = rust-toolchain-core;
          };
        in
        rec {
          servicepoint-simulator = naersk'.buildPackage rec {
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
                lzma
              ]
              ++ (lib.optionals pkgs.stdenv.isLinux (
                with pkgs;
                [
                  libxkbcommon
                  libGL

                  # WINIT_UNIX_BACKEND=wayland
                  wayland

                  # WINIT_UNIX_BACKEND=x11
                  xorg.libXcursor
                  xorg.libXrandr
                  xorg.libXi
                  xorg.libX11
                  xorg.libX11.dev
                ]
              ));

            postInstall = ''
              wrapProgram $out/bin/servicepoint-simulator \
                --suffix LD_LIBRARY_PATH : ${lib.makeLibraryPath buildInputs}
            '';
          };

          default = servicepoint-simulator;
        }
      );

      legacyPackages = packages;

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages."${system}";
          rust-toolchain = pkgs.symlinkJoin {
            name = "rust-toolchain";
            paths = with pkgs; [
              (make-rust-toolchain-core pkgs)
              rustfmt
              clippy
              cargo-expand
            ];
          };
        in
        {
          default = pkgs.mkShell rec {
            inputsFrom = [ self.packages.${system}.default ];
            packages = [ rust-toolchain ];
            LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath (builtins.concatMap (d: d.buildInputs) inputsFrom)}";
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };
        }
      );

      formatter = forAllSystems (system: nixpkgs.legacyPackages."${system}".nixfmt-rfc-style);
    };
}
