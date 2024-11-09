{
  description = "Flake for servicepoint-simulator";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
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
    in
    rec {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages."${system}";
          naersk' = pkgs.callPackage naersk {
            cargo = pkgs.cargo;
            rustc = pkgs.rustc;
          };
        in
        rec {
          servicepoint-simulator = naersk'.buildPackage rec {
            src = ./.;
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

            #postFixup = ''
            #  patchelf $out/bin/servicepoint-simulator --add-rpath ${pkgs.lib.makeLibraryPath buildInputs}
            #'';

            #postInstall = ''
            #  patchelf $out/bin/servicepoint-simulator --add-rpath ${pkgs.lib.makeLibraryPath buildInputs}
            #'';
          };

          default = servicepoint-simulator;
        }
      );

      legacyPackages = packages;

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages."${system}";
        in
        {
          default = pkgs.mkShell rec {
            inputsFrom = [ self.packages.${system}.default ];
            packages = with pkgs; [
              rustfmt
              cargo-expand
            ];
           # LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath (
           #   builtins.concatMap (d: d.runtimeDependencies) inputsFrom
           # )}";
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };
        }
      );

      formatter = forAllSystems (system: nixpkgs.legacyPackages."${system}".nixfmt-rfc-style);
    };
}
