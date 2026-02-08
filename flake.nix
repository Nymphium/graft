{
  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/*";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        rustPackages = with pkgs; [
          cargo
          rustc
          rust-analyzer
          rustfmt
        ];

        formatter = pkgs.nixfmt-tree;

        devShells.default = pkgs.mkShellNoCC {
          packages = rustPackages  ++ [
            pkgs.actionlint

            pkgs.nil
            formatter
          ];
        };
      in
      {
        legacyPackages = pkgs;
        inherit formatter devShells;
      }
    );
}
