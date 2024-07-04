{
  description = ''
    A nix flake for the essential debugger.
  '';

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
  };

  outputs = inputs:
    let
      overlays = [
        inputs.self.overlays.default
      ];
      perSystemPkgs = f:
        inputs.nixpkgs.lib.genAttrs (import inputs.systems)
          (system: f (import inputs.nixpkgs { inherit overlays system; }));
    in
    {
      overlays = {
        essential-debugger = import ./overlay.nix { };
        default = inputs.self.overlays.essential-debugger;
      };

      packages = perSystemPkgs (pkgs: {
        essential-debugger = pkgs.essential-debugger;
        default = inputs.self.packages.${pkgs.system}.essential-debugger;
      });

      devShells = perSystemPkgs (pkgs: {
        essential-debugger-dev = pkgs.callPackage ./shell.nix { };
        default = inputs.self.devShells.${pkgs.system}.essential-debugger-dev;
      });

      apps = perSystemPkgs (pkgs: {
        debugger = {
          type = "app";
          program = "${pkgs.essential-debugger}/bin/essential-debugger";
        };
        default = inputs.self.apps.${pkgs.system}.debugger;
      });

      formatter = perSystemPkgs (pkgs: pkgs.nixpkgs-fmt);
    };
}
