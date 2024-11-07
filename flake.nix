{
  description = "Flake for servicepoint-simulator";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
  };

  outputs =
    inputs@{ self, nixpkgs }:
    rec {
      packages.hello = nixpkgs.rustPlatform.buildRustPackage rec {
        pname = "servicepoint-simulator";
        version = "0.0.1";

        src = [ ]; # TODO: src, Cargo.toml etc

        buildInputs = [

        ];
        nativeBuildInputs = with nixpkgs.legacyPackages.x86_64-linux; [ pkgconfig ];
        cargoSha256 = "sha256-0hfmV4mbr3l86m0X7EMYTOu/b+BjueVEbbyQz0KgOFY=";

        meta = with nixpkgs.stdenv.lib; {
          homepage = "";
          description = "";
          #license = licenses.gplv3;
        };

        legacyPackages = packages;

        defaultPackage = packages.hello;
      };

      devShells.x86_64-linux.default = import ./shell.nix { pkgs = nixpkgs.legacyPackages.x86_64-linux; };

      formatter = {
        x86_64-linux = nixpkgs.legacyPackages.x86_64-linux.nixfmt-rfc-style;
        aarch64-linux = nixpkgs.legacyPackages.aarch64-linux.nixfmt-rfc-style;
      };
    };
}
