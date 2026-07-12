{
  description = "Losange: A simple Stremio client for GNOME";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});

      mkLosange =
        pkgs:
        pkgs.callPackage ./nix/package.nix {
          revision = self.shortRev or self.dirtyShortRev or "dirty";
          craneLib = crane.mkLib pkgs;
        };
    in
    {
      overlays.default = final: prev: {
        losange = mkLosange prev;
      };

      packages = forAllSystems (pkgs: rec {
        losange = mkLosange pkgs;
        default = losange;
      });

      devShells = forAllSystems (pkgs: {
        default = pkgs.callPackage ./nix/devShell.nix {
          losange = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
        };
      });

      formatter = forAllSystems (pkgs: pkgs.nixfmt-tree);
    };
}
