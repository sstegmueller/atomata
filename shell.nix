{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.rustc
    pkgs.cargo
    pkgs.xorg.libX11
    pkgs.xorg.libXrandr
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.libxkbcommon
    pkgs.libGL
    pkgs.libGLU
    pkgs.fontconfig
    pkgs.wayland
    pkgs.nodejs
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
}