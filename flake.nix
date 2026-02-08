{
  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/*";
    flake-utils.url = "github:numtide/flake-utils";

    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      gitignore,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        rustBin = pkgs.rust-bin.stable.latest.default;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustBin;
          rustc = rustBin;
        };

        graft = pkgs.callPackage ./. {
          inherit (rustPlatform) buildRustPackage;
          inherit (gitignore.lib) gitignoreFilterWith;
        };

        gen-supported-languages = pkgs.writeShellApplication {
          name = "gen-supported-languages";
          runtimeInputs = [
            graft
            pkgs.coreutils
          ];
          text = ''
            cat <<'EOL' > docs/SUPPORTED_LANGUAGES.md
            Supported Languages
            ===

            The following languages are currently supported by Graft:
            EOL
            graft --list-languages >> docs/SUPPORTED_LANGUAGES.md
            echo "Generated docs/SUPPORTED_LANGUAGES.md"
          '';
        };

        rustPackages = [
          rustBin
          pkgs.rust-bin.stable.latest.rust-analyzer
        ];

        formatter = pkgs.nixfmt-tree;

        devShells.default = pkgs.mkShellNoCC {
          inputsFrom = [ graft ];
          packages = rustPackages ++ [
            pkgs.actionlint
            pkgs.nil
            formatter
          ];
        };
      in
      {
        packages = {
          default = graft;
          inherit gen-supported-languages;
        };
        legacyPackages = pkgs;
        inherit formatter devShells;
      }
    );
}
