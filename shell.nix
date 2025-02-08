{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell rec {
  name = "pre-commit-todo";

  buildInputs = with pkgs; [
    rustc # Rust compiler
    cargo # Rust package manager
    rustfmt # Rust code formatter
    clippy # Rust linter
    gcc # Required for crates needing C compilers
    pkg-config # Helps locate libraries like OpenSSL
    openssl # OpenSSL library for crates like openssl-sys
    git
    git-crypt
    stdenv.cc.cc.lib
    stdenv.cc.cc # jupyter lab needs
    pre-commit
  ];

  # Set the source path for Rust tooling (e.g., rust-analyzer)
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  pre-commit = pkgs.pre-commit;

  shellHook = ''
    export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"

    export RUST_BACKTRACE=1
    export CARGO_HOME=$HOME/.cargo
    export PATH=$CARGO_HOME/bin:$PATH
  '';
}
