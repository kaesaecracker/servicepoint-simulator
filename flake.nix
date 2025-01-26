{
  description = "Flake for servicepoint-simulator";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
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
      forAllSystems =
        f:
        lib.genAttrs supported-systems (
          system:
          f rec {
            pkgs = nixpkgs.legacyPackages.${system};
            inherit system;
          }
        );
    in
    rec {
      packages = forAllSystems (
        { pkgs, ... }:
        rec {
          servicepoint-simulator = import ./servicepoint-simulator.nix {
            inherit nix-filter pkgs;
            naersk' = pkgs.callPackage naersk { };
          };
          default = servicepoint-simulator;
        }
      );

      legacyPackages = packages;

      devShells = forAllSystems (
        {
          pkgs,
          system,
        }:
        {
          default = pkgs.mkShell rec {
            inputsFrom = [ self.packages.${system}.default ];
            packages = [
              pkgs.gdb
              (pkgs.symlinkJoin {
                name = "rust-toolchain";
                paths = with pkgs; [
                  rustc
                  cargo
                  rustPlatform.rustcSrc
                  rustfmt
                  clippy
                  cargo-expand
                ];
              })
            ];
            LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath (builtins.concatMap (d: d.buildInputs) inputsFrom)}";
            NIX_LD_LIBRARY_PATH = LD_LIBRARY_PATH;
            NIX_LD = pkgs.stdenv.cc.bintools.dynamicLinker;
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };
        }
      );

      formatter = forAllSystems ({ pkgs, ... }: pkgs.nixfmt-rfc-style);
    };
}
