{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
  }:
    utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};
      in {
        packages = rec { 
          default = tarts;
          tarts = 
            let
              manifest = pkgs.lib.importTOML ./Cargo.toml;
            in
            pkgs.rustPlatform.buildRustPackage {
              pname = manifest.package.name;
              version = manifest.package.version;

              src = pkgs.lib.cleanSource ./.;
              cargoLock.lockFile = ./Cargo.lock;
            };
        };
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            rustPackages.clippy
            bacon
			rust-analyzer
          ];
          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
        };
      }
    );
}
