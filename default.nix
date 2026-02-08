{
  pkgs,
  lib,
  buildRustPackage,
  gitignoreFilterWith,
}:
buildRustPackage {
  pname = "graft";
  version = "0.1.0";
  src =
    let
      src = ./.;
      filter =
        basePath:
        gitignoreFilterWith {
          inherit basePath;
          extraRules = ''
            *.nix
            flake.lock
            .github
            examples
            tests
            README.md
            SUPPORTED_LANGUAGES.md
          '';
        };
    in
    lib.sources.cleanSourceWith {
      filter = filter src;
      inherit src;
      name = "filtered-source";
    };
  cargoLock = {
    lockFile = ./Cargo.lock;
  };
  nativeBuildInputs = [ pkgs.pkg-config ];
  buildInputs = [ pkgs.tree-sitter ];

  doCheck = true;
}
