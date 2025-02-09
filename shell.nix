{ pkgs ? import <nixpkgs> {}, overlays ? [] }:

let
  rust-overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust-overlay ]; };

  rust = pkgs.rust-bin.stable.latest.default.override {
    extensions = [ "rust-src" "cargo" "rustc" "clippy" "rustfmt" ];
  };
in

pkgs.mkShell {
  name = "comment-parser";

  buildInputs = with pkgs; [
    rust
    rust-analyzer
    pkg-config
    openssl
    git
    pre-commit
    lldb
    llvmPackages.libllvm
    gcc
    zlib
    zlib.out
  ];

  shellHook = ''
    export RUST_BACKTRACE=1
    export CARGO_HOME=$HOME/.cargo
    export PATH=$CARGO_HOME/bin:$PATH
    export RUST_SRC_PATH="${rust}/lib/rustlib/src/rust/library"

    export LD_LIBRARY_PATH=${pkgs.zlib}/lib:$LD_LIBRARY_PATH
  '';
}
