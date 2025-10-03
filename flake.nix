{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, utils, rust-overlay, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable."1.89.0".default;
      in {
        formatter = pkgs.nixfmt-tree;

        packages = rec {
          default = tarts;
          tarts =
            let
              manifest = pkgs.lib.importTOML ./Cargo.toml;
            in pkgs.rustPlatform.buildRustPackage {
              pname = manifest.package.name;
              version = manifest.package.version;

              inherit rustToolchain;

              src = pkgs.lib.cleanSource ./.;
              cargoLock.lockFile = ./Cargo.lock;
              meta.mainProgram = "tarts";
            };
        };

        devShells.default = pkgs.mkShell {
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
      });
}
