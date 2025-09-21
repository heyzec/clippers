{
  inputs = {nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";};

  outputs = {
    self,
    nixpkgs,
    ...
  }: let
    pkgsFor = system:
      import nixpkgs {inherit system;};

    targetSystems = ["x86_64-linux" "aarch64-darwin"];
  in {
    devShells = nixpkgs.lib.genAttrs targetSystems (system: let
      pkgs = pkgsFor system;
    in {
      default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          # Compilers
          rustc
          cargo

          # Tools
          rust-analyzer
          rustfmt
        ];
      };
    });
  };
}
