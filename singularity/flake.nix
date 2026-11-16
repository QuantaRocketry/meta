{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ ];
        pkgs = import nixpkgs { inherit system overlays; };
        nativeBuildInputs = with pkgs; [
          pkg-config
          gobject-introspection
          cargo
          cargo-tauri
          rust-analyzer
          nodejs
          just
          pnpm
        ];
        buildInputs = with pkgs; [
          at-spi2-atk
          atkmm
          cairo
          gdk-pixbuf
          glib
          gtk3
          harfbuzz
          librsvg
          libsoup_3
          pango
          webkitgtk_4_1
          openssl
          udev
        ];
      in with pkgs; {
        devShells.default = mkShell { inherit buildInputs nativeBuildInputs; };
      });
}
