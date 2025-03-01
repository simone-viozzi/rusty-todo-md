{ pkgs ? import <nixpkgs> {}, overlays ? [] }:

let
  rust-overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust-overlay ]; };

  rust = pkgs.rust-bin.stable.latest.default.override {
    extensions = [ "rust-src" "cargo" "rustc" "clippy" "rustfmt" ];
  };

  pythonPackages = pkgs.python311Packages;
in

pkgs.mkShell {
  name = "comment-parser";
  venvDir = "./.venv";

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
    pythonPackages.python
    pythonPackages.pyzmq # Adding pyzmq explicitly
    pythonPackages.venvShellHook
    pythonPackages.pip
    pythonPackages.ruff
    pythonPackages.click
    pythonPackages.pathspec
    pythonPackages.tqdm
    pythonPackages.pytest
    pre-commit
  ];

  postVenvCreation = ''
    unset SOURCE_DATE_EPOCH
  '';

  pre-commit = pkgs.pre-commit;

  postShellHook = ''
    export RUST_BACKTRACE=1
    export CARGO_HOME=$HOME/.cargo
    export PATH=$CARGO_HOME/bin:$PATH
    export RUST_SRC_PATH="${rust}/lib/rustlib/src/rust/library"

    export LD_LIBRARY_PATH=${pkgs.zlib}/lib:$LD_LIBRARY_PATH

    # allow pip to install wheels
    unset SOURCE_DATE_EPOCH

    pip install --upgrade wheel setuptools
    export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
  '';
}
