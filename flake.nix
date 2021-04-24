{
  description = "avmetadata";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nmattia/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixCargoIntegration = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, utils, naersk, fenix, nixCargoIntegration }:
    nixCargoIntegration.lib.makeOutputs {
      root = ./.;
      overrides = {
        pkgs = common: prev: let
          fenix' = fenix.packages."${common.system}";
        in {
          overlays = prev.overlays ++ [
            (final: prevv: {
              rustc = fenix'.latest.rustc;
            })
          ];
        };

        shell = common: prev: let
          pkgs = nixpkgs.legacyPackages."${common.system}";
          fenix' = fenix.packages."${common.system}";
        in {
          packages = prev.packages ++ [ fenix'.latest.toolchain ];
        };
      };
    };
}
