{
  rustPlatform,
  lib,
  rev ? "dirty",
  ...
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
in
  rustPlatform.buildRustPackage rec {
    pname = "clippers";
    version = "${cargoToml.package.version}-${rev}";

    src = lib.fileset.toSource {
      root = ./.;
      fileset = lib.fileset.intersection (lib.fileset.fromSource (lib.sources.cleanSource ./.)) (
        lib.fileset.unions [
          ./src
          ./Cargo.toml
          ./Cargo.lock
        ]
      );
    };

    cargoLock.lockFile = ./Cargo.lock;
  }
