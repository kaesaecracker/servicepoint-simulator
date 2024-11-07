{
  description = "Flake for servicepoint-simulator";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
  };

  outputs =
    inputs@{ self, nixpkgs }:
    let
      lib = nixpkgs.lib;
      forAllSystems = lib.genAttrs lib.systems.flakeExposed;
    in
    rec {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages."${system}";
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "servicepoint-simulator";
            version = "0.0.1";
            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };

            src = ./.;

            nativeBuildInputs = with pkgs; [ pkg-config ];

            buildInputs =
              with pkgs;
              [
                xe
                lzma
              ]
              ++ (lib.optionals pkgs.stdenv.isLinux (
                with pkgs;
                [
                  xorg.libxkbfile
                  wayland
                  libxkbcommon
                ]
              ));

            meta = with lib; {
              homepage = "";
              description = "";
              license = licenses.gpl3;
            };
          };
        }
      );

      legacyPackages = packages;

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages."${system}";
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.default ];
            packages = with pkgs; [
              rustfmt
              cargo-expand
            ];
            #LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath buildInputs}";
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };
        }
      );

      formatter = forAllSystems (system: nixpkgs.legacyPackages."${system}".nixfmt-rfc-style);
    };
}
