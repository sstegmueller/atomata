{
  inputs = {
    nixpkgs = { url = "github:nixos/nixpkgs/nixos-unstable"; };
    rust-overlay = { url = "github:oxalica/rust-overlay"; };
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            cargo
            nodejs
            xorg.libX11
            xorg.libXrandr
            xorg.libXcursor
            xorg.libXi
            libxkbcommon
            libGL
            libGLU
            fontconfig
            wayland
            wasm-pack
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
              targets = [ "wasm32-unknown-unknown" ];
            })
          ];

          shellHook = ''
            export PATH=$PATH:~/.cargo/bin
            export RUST_BACKTRACE=1
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [
              pkgs.libGL
              pkgs.libxkbcommon
              pkgs.wayland
              pkgs.xorg.libX11
              pkgs.xorg.libXcursor
              pkgs.xorg.libXi
              pkgs.xorg.libXrandr
            ]}
          '';

          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
        };
    }
  );
}