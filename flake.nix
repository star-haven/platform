{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    treefmt-nix.url = "github:numtide/treefmt-nix";

    devDB.url = "github:hermann-p/nix-postgres-dev-db";
    devDB.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      imports = [
        inputs.treefmt-nix.flakeModule
        ./nix/flake-module.nix
      ];
      perSystem = { config, self', pkgs, system, ... }: let
        db = inputs.devDB.outputs.packages.${system};
      in {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            inputs.rust-overlay.overlays.default
          ];
        };

        treefmt.config = {
          projectRootFile = "flake.nix";
          programs = {
            nixpkgs-fmt.enable = true;
            rustfmt.enable = true;
            leptosfmt.enable = true;
          };
        };

        packages.default = self'.packages.star-haven-platform;

        devShells.default = pkgs.mkShell {
          inputsFrom = [
            config.treefmt.build.devShell
            self'.devShells.star-haven-platform
          ];
          nativeBuildInputs = with pkgs; [
            just
            cargo-watch
            postgresql
            sea-orm-cli
            db.start-database
            db.stop-database
            db.psql-wrapped
          ];
          shellHook = ''
            export PG_ROOT=$(git rev-parse --show-toplevel)
            start-database
            export DATABASE_URL="postgres://$USER@localhost/dev"
          '';
        };
      };
    };
}
