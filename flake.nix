{
  description = "Flake for servicepoint-simulator";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
  };

  outputs =
    inputs@{ self, nixpkgs }:
    let
      servicepoint-simulator = nixpkgs.legacyPackages.x86_64-linux.rustPlatform.buildRustPackage rec {
        pname = "servicepoint-simulator";
        version = "0.0.1";

        src = ./.; # TODO: src, Cargo.toml etc

        buildInputs = [

        ];
        nativeBuildInputs = with nixpkgs.legacyPackages.x86_64-linux; [ pkg-config ];
        #cargoSha256 = "sha256-0hfmV4mbr3l86m0X7EMYTOu/b+BjueVEbbyQz0KgOFY=";
        cargoLock.lockFile = ./Cargo.lock;

        meta = with nixpkgs.stdenv.lib; {
          homepage = "";
          description = "";
          #license = licenses.gplv3;
        };

      };
    in
    rec {
      packages.x86_64-linux.default = servicepoint-simulator;

      legacyPackages = packages;

      devShells.x86_64-linux.default = import ./shell.nix { pkgs = nixpkgs.legacyPackages.x86_64-linux; };

      formatter = {
        x86_64-linux = nixpkgs.legacyPackages.x86_64-linux.nixfmt-rfc-style;
        aarch64-linux = nixpkgs.legacyPackages.aarch64-linux.nixfmt-rfc-style;
      };
    };
}
